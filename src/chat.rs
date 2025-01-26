//! Maintain a chat session with LLMs in your terminal.
//!
//! Run `yap chat --new` to start a new chat.
//!
//! Then, chat with the LLM;
//!
//! ```bash
//! $ yap chat 'What is your favorite color?'
//! ```
//!
//! To clear out the chat history, pass the `--new` flag again.

use crate::{
    config::ConfigFile,
    constants, db,
    err::{Error, Oops},
    openai::{self, CompletionPayload, Content, Message, PayloadOpts, Role},
};
use log::debug;
use uuid::Uuid;

/// Entrypoint for `yap chat`. If `new` is set, we will begin a new chat
/// session.
pub fn chat(
    open_ai: &openai::OpenAI,
    prompt: &[String],
    new: bool,
    resume: Option<&Uuid>,
) -> Result<(), Error> {
    debug!("Chatting with prompt {prompt:?}");

    if resume.is_some() && new {
        return Err(Error::default().wrap(Oops::ChatError).because(
            "Cannot specify --new and --resume together.".to_string(),
        ));
    }

    let chat_id = if let Some(id) = resume {
        let id = *id;
        db::set_chat_id(&id)?;
        id
    } else if new {
        let id = Uuid::new_v4();
        db::set_chat_id(&id)?;
        id
    } else {
        db::get_active_chat()?.map_or_else(
            || {
                // Create a new chat if there is no active one.
                let id = Uuid::new_v4();
                db::set_chat_id(&id)?;
                Ok(id)
            },
            Ok,
        )?
    };

    if prompt.is_empty() && new {
        debug!("prompt is empty, but --new was passed. Exiting from chat early because a new and empty chat was started.");
        return Ok(());
    } else if prompt.is_empty() {
        return Err(Error::default()
            .wrap(Oops::ChatError)
            .because("Prompt is empty!".to_string()));
    }

    resume_chat(open_ai, &chat_id, prompt)
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
