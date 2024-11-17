//! Maintain a chat session with LLMs in your terminal.

use crate::{
    db,
    err::{Error, Oops},
    openai::{self, CompletionPayload, Content, Message, Model, Role},
};
use log::debug;
use std::env::{self, VarError};
use uuid::Uuid;

pub fn chat(
    open_ai: &openai::OpenAI,
    prompt: &Option<Vec<String>>,
) -> Result<(), Error> {
    debug!("Chatting with prompt {prompt:?}");
    let maybe_id = match env::var("YAP_CHAT_ID") {
        Ok(id) => Ok(Some(Uuid::parse_str(&id).map_err(|e| {
            Error::default()
                .wrap(Oops::ChatError)
                .because(format!("Could not parse UUID from {id}: {e}"))
        })?)),
        Err(e) => match e {
            VarError::NotUnicode(_) => Err(Error::default()
                .wrap(Oops::ChatError)
                .because("$YAP_CHAT_ID is not unicode".into())),
            VarError::NotPresent => Ok(None),
        },
    }?;
    match (maybe_id, prompt) {
        (Some(id), Some(prompt)) => resume_chat(open_ai, &id, prompt),
        (None, Some(_)) => {
            eprintln!(
                r#"Error: no chat in progress! Start a new chat with `eval "$(yap chat)""#
            );
            // Silently exit non-zero
            Err(Error::default())
        }
        (Some(_), None) => {
            create_chat();
            Ok(())
        }
        (None, None) => {
            create_chat();
            Ok(())
        }
    }
}

fn resume_chat(
    open_ai: &openai::OpenAI,
    id: &Uuid,
    prompt: &[String],
) -> Result<(), Error> {
    let mut messages = db::get_chat(id)?;
    messages.push(Message::new(Role::User, prompt.join(" ")));
    let reply = openai::chat(
        open_ai,
        &CompletionPayload {
            messages: messages.clone(),
            model: Model::Gpt4o,
        },
    )?;
    messages.push(reply.choices[0].message.clone());
    db::save_chat(id, &messages)?;

    match reply.choices[0].message.parse()? {
        Content::Normal(msg) => println!("{msg}"),
        Content::Refusal(msg) => eprintln!("{msg}"),
    };
    Ok(())
}

/// Prints `export YAP_CHAT_ID=<uuid>` to STDOUT, which effectively creates a
/// new chat. Intended usage is `eval "$(yap chat)"`.
fn create_chat() {
    let new_id = Uuid::new_v4().to_string();
    println!(
        r##"# hint: run `eval "$(yap chat)"` to start a new chat.
export YAP_CHAT_ID='{new_id}'"##
    )
}
