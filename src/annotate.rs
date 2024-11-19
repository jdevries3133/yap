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
use std::{fs::read_to_string, path::PathBuf};

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
    line_number: u64,
    content: String,
}

pub fn annotate(
    open_ai: &OpenAI,
    prompt: &str,
    file: &PathBuf,
    line_start: usize,
    line_end: Option<usize>,
) -> Result<(), Error> {
    let file_contents = read_to_string(file).map_err(|e| {
        Error::default().wrap(Oops::AnnotateError).because(format!(
            "Error while opening the file to annotate ({file:?}): {e}"
        ))
    })?;
    let iter = file_contents.split("\n");
    let target_contents = iter
        .skip(line_start)
        .take(line_end.map(|v| v - line_start).unwrap_or(usize::MAX))
        .collect::<Vec<&str>>()
        .join("\n");
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
    println!("{annotation:?}");
    Ok(())
}
