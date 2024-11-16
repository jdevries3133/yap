//! Write completion for prompts to `STDIN` to `STDOUT`.

use crate::{
    config::get_system_prompt_for_completion,
    err::{Error, Oops},
    openai::{chat, CompletionPayload, Content, Message, Model, OpenAI, Role},
};
use std::io::{self, Read};

pub fn complete(open_ai: &OpenAI) -> Result<(), Error> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).map_err(|e| {
        Error::default()
            .wrap(Oops::CompletionError)
            .wrap(Oops::StdinReadError)
            .because(e.kind().to_string())
    })?;

    let system_prompt = get_system_prompt_for_completion().map_err(|e| {
        e.wrap(Oops::CompletionError)
            .because("could not get system prompt for completion".into())
    })?;

    let payload = CompletionPayload {
        model: Model::Gpt4oMini,
        messages: vec![
            Message::new(Role::System, system_prompt),
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
