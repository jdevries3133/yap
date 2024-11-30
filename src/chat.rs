//! Maintain a chat session with LLMs in your terminal.
//!
//! Run `eval "$(yap chat)" to start a new session. Under the hood, this
//! simply redefines `YAP_CHAT_HISTORY_FILE` with a new UUID;
//!
//! ```bash
//! $ yap chat
//! # hint: run `eval "$(yap chat)"` to start a new chat.
//! # Or, copy and paste the line below into your shell.
//! export YAP_CHAT_HISTORY_FILE='775a04f6-071e-4d7e-929b-043ff1260eed'
//! ```
//!
//! You can feel free to hack on this interface to create named chats. See
//! [crate::db] for details.
//!
//! Once `YAP_CHAT_HISTORY_FILE` is defined in your environment, you can talk
//! to LLMs in your terminal, and your conversation history is persisted;
//!
//! ```bash
//! $ yap chat 'What is your favorite color?'
//! ```

use crate::{
    config::ConfigFile,
    constants, db,
    err::{Error, Oops},
    openai::{self, CompletionPayload, Content, Message, PayloadOpts, Role},
};
use log::debug;
use std::env::{self, VarError};
use uuid::Uuid;

/// Entrypoint for `yap chat`.
///
/// If the prompt is empty, we will provide a new chat ID, which can be
/// used via `eval "$(yap chat)"`.
///
/// If we have a prompt and a `YAP_CHAT_HISTORY_FILE` is available, we will
/// respond to the chat, and then save the conversation. If a chat history is
/// available associated with the `YAP_CHAT_HISTORY_FILE`, we will append the
/// current prompt to the conversation so far before sending it off to OpenAI.
///
/// If we have a prompt, but `YAP_CHAT_HISTORY_FILE` is not defined, we will
/// return an error.
pub fn chat(
    open_ai: &openai::OpenAI,
    prompt: &Option<Vec<String>>,
) -> Result<(), Error> {
    debug!("Chatting with prompt {prompt:?}");
    let maybe_id = match env::var("YAP_CHAT_HISTORY_FILE") {
        Ok(id) => Ok(Some(Uuid::parse_str(&id).map_err(|e| {
            Error::default()
                .wrap(Oops::ChatError)
                .because(format!("Could not parse UUID from {id}: {e}"))
        })?)),
        Err(e) => match e {
            VarError::NotUnicode(_) => Err(Error::default()
                .wrap(Oops::ChatError)
                .because("$YAP_CHAT_HISTORY_FILE is not unicode".into())),
            VarError::NotPresent => Ok(None),
        },
    }?;
    match (maybe_id, prompt) {
        (Some(id), Some(prompt)) => resume_chat(open_ai, &id, prompt),
        (None, Some(_)) => {
            eprintln!(
                r#"Error: no chat in progress! Start a new chat with `eval "$(yap chat)"`"#
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

/// If available, load the chat history associated with `id`, append the
/// prompt to chat history, send to OpenAI, print the response, and then
/// persist the new chat history.
fn resume_chat(
    open_ai: &openai::OpenAI,
    id: &Uuid,
    prompt: &[String],
) -> Result<(), Error> {
    let mut messages = db::get_chat(id)?;
    if messages.is_empty() {
        let system_prompt = ConfigFile::ChatSystemPrompt
            .load()
            .map_err(|e| {
                e.wrap(Oops::ChatError)
                    .because("Could not load system prompt during chat".into())
            })?
            .map_or(constants::DEFAULT_CHAT_PROMPT.to_string(), |p| p.clone());
        messages.push(Message::new(Role::System, system_prompt));
    }
    messages.push(Message::new(Role::User, prompt.join(" ")));
    let reply = openai::chat(
        open_ai,
        &CompletionPayload::new(
            open_ai,
            messages.clone(),
            PayloadOpts::default(),
        ),
    )?;
    messages.push(reply.choices[0].message.clone());
    db::save_chat(id, &messages)?;

    match reply.choices[0].message.parse()? {
        Content::Normal(msg) => println!("{msg}"),
        Content::Refusal(msg) => eprintln!("{msg}"),
    };
    Ok(())
}

/// Prints `export YAP_CHAT_HISTORY_FILE=<uuid>` to STDOUT, which effectively
/// creates a new chat. Intended usage is `eval "$(yap chat)"`.
fn create_chat() {
    let new_id = Uuid::new_v4().to_string();
    println!(
        r##"# hint: run `eval "$(yap chat)"` to start a new chat.
# Or, copy and paste the line below into your shell.
export YAP_CHAT_HISTORY_FILE='{new_id}'"##
    )
}
