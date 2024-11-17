//! Write completion for prompts to `STDIN` to `STDOUT`.

use crate::{
    config::ConfigFile,
    constants,
    err::{Error, Oops},
    openai::{chat, CompletionPayload, Content, Message, Model, OpenAI, Role},
};
use std::io::{self, Read};

/// Entrypoint for `yap complete`
///
/// Read into `STDIN`, and print completion to `STDOUT`. Load the system
/// prompt from ~/.config/yap/complete_system_prompt.txt` if available,
/// or else use the default prompt from
/// [crate::constants::DEFAULT_COMPLETION_PROMPT].
pub fn complete(open_ai: &OpenAI) -> Result<(), Error> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).map_err(|e| {
        Error::default()
            .wrap(Oops::CompletionError)
            .wrap(Oops::StdinReadError)
            .because(e.kind().to_string())
    })?;

    let system_prompt_maybe =
        ConfigFile::CompleteSystemPrompt.load().map_err(|e| {
            e.wrap(Oops::CompletionError)
                .because("could not get system prompt for completion".into())
        })?;

    let system_prompt = system_prompt_maybe
        .as_ref()
        .map_or(constants::DEFAULT_COMPLETION_PROMPT, |s| s);

    let payload = CompletionPayload {
        model: Model::Gpt4oMini,
        messages: vec![
            Message::new(Role::System, system_prompt.to_string()),
            Message::new(Role::User, input),
        ],
    };
    let response = chat(open_ai, &payload)?;
    let content = response.choices[0].message.parse()?;
    match content {
        Content::Normal(c) => println!("{}", c),
        Content::Refusal(r) => eprintln!("{}", r),
    };
    Ok(())
}
