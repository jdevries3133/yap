//! (not yet implemented) annotate a source-code file based on a prompt
//! request.

use crate::{
    constants,
    err::{Error, Oops},
    openai::{
        chat, CompletionPayload, Content, Message, OpenAI, PayloadOpts,
        ResponseFormat, Role,
    },
};
use log::debug;
use serde::Deserialize;
use serde_json::{from_str, json, Value};
use std::{
    fs::{read_to_string, File},
    io::{BufRead, BufReader, Cursor, Write},
    path::PathBuf,
};

fn get_json_schema() -> Value {
    json!({
      "name": "source_file_annotations",
      "schema": {
        "type": "object",
        "properties": {
          "annotations": {
            "type": "array",
            "description": "A list of annotations related to the source file.",
            "items": {
              "type": "object",
              "properties": {
                "line_number": {
                  "type": "number",
                  "description": "The line number in the source file where the annotation applies."
                },
                "content": {
                  "type": "string",
                  "description": "The content of the annotation."
                }
              },
              "required": ["line_number", "content"],
              "additionalProperties": false
            }
          }
        },
        "required": ["annotations"],
        "additionalProperties": false
      },
      "strict": true
    })
}

#[derive(Debug, Deserialize)]
struct AnnotationResponse {
    annotations: Vec<Annotation>,
}

#[derive(Debug, Deserialize)]
struct Annotation {
    line_number: usize,
    content: String,
}

pub fn annotate(
    open_ai: &OpenAI,
    prompt: &str,
    file: &PathBuf,
    comment_prefix: &Option<String>,
    comment_suffix: &Option<String>,
) -> Result<(), Error> {
    let file_contents = read_to_string(file).map_err(|e| {
        Error::default().wrap(Oops::AnnotateError).because(format!(
            "Error while opening the file to annotate ({file:?}): {e}"
        ))
    })?;
    let file_type_info = FileTypeInfo::new(
        comment_prefix.as_ref().map(|s| s.as_str()),
        comment_suffix.as_ref().map(|s| s.as_str()),
    );
    let iter = file_contents.split("\n");
    let target_contents = iter.collect::<Vec<&str>>().join("\n");
    let payload = CompletionPayload::new(
        open_ai,
        vec![
            Message::new(
                Role::System,
                constants::DEFAULT_ANNOTATE_PROMPT.into(),
            ),
            Message::new(Role::User, prompt.into()),
            Message::new(Role::User, target_contents),
        ],
        PayloadOpts {
            response_format: ResponseFormat::JsonSchema {
                json_schema: get_json_schema(),
            },
        },
    );
    let response = chat(open_ai, &payload).map_err(|e| {
        e.wrap(Oops::AnnotateError)
            .because("Error after sending annotation payload to OpenAI".into())
    })?;
    let message = &response.choices[0].message;
    let content = message.parse().map_err(|e| {
        e.wrap(Oops::AnnotateError)
            .because("Could not parse OpenAi response content".into())
    })?;
    let annotation_str = match content {
        Content::Normal(c) => Ok(c),
        Content::Refusal(r) => {
            Err(Error::default().wrap(Oops::AnnotateError).because(format!(
            "OpenAI sent a refusal in response to your annotation request: {r}"
        )))
        }
    }?;
    let annotation: AnnotationResponse =
        from_str(annotation_str).map_err(|e| {
            debug!("Bad response content: {annotation_str}");
            Error::default().wrap(Oops::AnnotateError).because(format!(
                "Failed to deserialize annotation string into annotations: {e}"
            ))
        })?;

    debug!("Applying annotations {:?}", annotation.annotations);

    let cursor = Cursor::new(file_contents);
    let reader = BufReader::new(cursor);
    let mut write_buffer = vec![];
    apply_annotations(
        reader,
        &mut write_buffer,
        annotation.annotations,
        file_type_info,
    )
    .map_err(|e| {
        e.wrap(Oops::AnnotateError)
            .because(format!("Error occurred while annotating {file:?}"))
    })?;

    File::create(file)
        .map_err(|e| {
            Error::default().wrap(Oops::AnnotateError).because(format!(
                "Could not open annotation target ({file:?}) for writing: {e}"
            ))
        })?
        .write(&write_buffer)
        .map_err(|e| {
            Error::default().wrap(Oops::AnnotateError).because(format!(
                "Error while writing annotations into {file:?}: {e}"
            ))
        })?;

    Ok(())
}

#[derive(Clone, Copy)]
struct FileTypeInfo<'a> {
    comment_suffix: &'a str,
    comment_prefix: &'a str,
}

impl<'a> FileTypeInfo<'a> {
    fn new(prefix: Option<&'a str>, suffix: Option<&'a str>) -> Self {
        Self {
            comment_prefix: prefix.as_ref().map_or("// ", |v| v),
            comment_suffix: suffix.as_ref().map_or("", |v| v),
        }
    }
}

fn apply_annotations<R: BufRead, W: Write>(
    reader: R,
    writer: &mut W,
    mut annotations: Vec<Annotation>,
    file_type_info: FileTypeInfo,
) -> Result<(), Error> {
    annotations.sort_by_key(|a| a.line_number);

    let mut annotations_iter = annotations.into_iter();
    let mut current_annotation = annotations_iter.next();

    for (line_number, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| {
            Error::default().wrap(Oops::AnnotateError).because(format!(
                "I/O error while reading file to annotate: {e}"
            ))
        })?;
        if let Some(annotation) = &current_annotation {
            if line_number + 1 == annotation.line_number {
                write!(
                    writer,
                    "{}\n{}\n",
                    yapify_annotation_content(
                        &annotation.content,
                        0,
                        file_type_info
                    ),
                    line
                )
                .map_err(|e| {
                    Error::default().wrap(Oops::AnnotateError).because(format!(
                        "Error while writing annotation into output: {e:?}"
                    ))
                })?;
                current_annotation = annotations_iter.next();
            } else {
                writeln!(writer, "{}", line).map_err(|e| Error::default().wrap(Oops::AnnotateError).because(
                        format!(
                            "Error while writing from reader to writer (lineno does not match): {e:?}"
                        )
                ))?;
            }
        } else {
            writeln!(writer, "{}", line).map_err(|e| Error::default().wrap(Oops::AnnotateError).because(
                    format!(
                        "Error while writing from reader to writer (no annotation): {e:?}"
                    )
            ))?;
        }
    }
    Ok(())
}

/// Transforms potentially multi-line content into;
///
/// ```plain
/// {' ' * left_padding}{prefix}yap :: {content}{suffix}
/// ```
fn yapify_annotation_content(
    content: &'_ str,
    left_padding: usize,
    file_type_info: FileTypeInfo,
) -> String {
    let mut output = String::with_capacity(content.len());
    for line in content.lines() {
        for _ in 0..left_padding {
            output.push(' ');
        }
        output.push_str(file_type_info.comment_prefix);
        output.push_str("yap :: ");
        output.push_str(line);
        output.push_str(file_type_info.comment_suffix);
        output.push('\n');
    }
    // Remove the trailing newline.
    output.pop();
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    fn typical_info() -> FileTypeInfo<'static> {
        FileTypeInfo::new(Some("// "), Some(""))
    }

    fn html_info() -> FileTypeInfo<'static> {
        FileTypeInfo::new(Some("<!-- "), Some(" -->"))
    }

    #[test]
    fn test_apply_annotation() {
        let input_data = "#!/bin/sh

echo 'hello world'"
            .to_string();

        let annotations = vec![Annotation {
            line_number: 3,
            content: r#"this will print "hello world" to STDOUT"#.into(),
        }];
        let expected_output = r##"#!/bin/sh

// yap :: this will print "hello world" to STDOUT
echo 'hello world'
"##;

        let reader = BufReader::new(Cursor::new(input_data));
        let mut output = Vec::new();
        let mut writer = Cursor::new(&mut output);

        apply_annotations(reader, &mut writer, annotations, typical_info())
            .unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, expected_output);
    }
    #[test]
    fn test_apply_annotation_out_of_order() {
        let input_data = "#!/bin/sh

echo 'hello world'

exit 1
"
        .to_string();

        let annotations = vec![
            Annotation {
            line_number: 5,
            content: r"Exit with non-zero status, indicating that an error has occurred.".into(),
            },
            Annotation {
            line_number: 3,
            content: r#"print "hello world" to STDOUT"#.into(),
        }];
        let expected_output = r##"#!/bin/sh

// yap :: print "hello world" to STDOUT
echo 'hello world'

// yap :: Exit with non-zero status, indicating that an error has occurred.
exit 1
"##;

        let reader = BufReader::new(Cursor::new(input_data));
        let mut output = Vec::new();
        let mut writer = Cursor::new(&mut output);

        apply_annotations(reader, &mut writer, annotations, typical_info())
            .unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, expected_output);
    }
    #[test]
    fn test_apply_annotation_multi_line() {
        let input_data = "// main.rs

value.as_ref().map(|i| i.as_str()).iter().reduce(String::new(), |a, v| {
    a.push(v);
    a
})
";
        let annotations = vec![Annotation {
            line_number: 3,
            content: "It does that\nIt does this\nIt does other thing".into(),
        }];

        let expected_output = "// main.rs

// yap :: It does that
// yap :: It does this
// yap :: It does other thing
value.as_ref().map(|i| i.as_str()).iter().reduce(String::new(), |a, v| {
    a.push(v);
    a
})
";
        let reader = BufReader::new(Cursor::new(input_data));
        let mut output = Vec::new();
        let mut writer = Cursor::new(&mut output);

        apply_annotations(reader, &mut writer, annotations, typical_info())
            .unwrap();

        let result = String::from_utf8(output).unwrap();
        println!("{}\n{}", result, expected_output);
        assert_eq!(result, expected_output);
    }
    #[test]
    fn test_apply_annotation_for_html_like_syntax() {
        let input_data = "<!-- This is a comment -->
<!DOCTYPE html>
<html>
<head>
    <title>Test Document</title>
</head>
<body>
    <h1>Hello World</h1>
</body>
</html>
"
        .to_string();

        let annotations = vec![
            Annotation {
                line_number: 2,
                content: "This comment provides context for the HTML document."
                    .into(),
            },
            Annotation {
                line_number: 8,
                content: "This is the main heading of the page.".into(),
            },
        ];

        let expected_output = r##"<!-- This is a comment -->
<!-- yap :: This comment provides context for the HTML document. -->
<!DOCTYPE html>
<html>
<head>
    <title>Test Document</title>
</head>
<body>
<!-- yap :: This is the main heading of the page. -->
    <h1>Hello World</h1>
</body>
</html>
"##;

        let reader = BufReader::new(Cursor::new(input_data));
        let mut output = Vec::new();
        let mut writer = Cursor::new(&mut output);

        apply_annotations(reader, &mut writer, annotations, html_info())
            .unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, expected_output);
    }
}
