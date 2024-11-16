//! Write completion for prompts to `STDIN` to `STDOUT`.

use crate::{
    err::{Error, Oops},
    openai::{CompletionPayload, Content, Message, Model, OpenAI, Role},
};
use std::io::{self, Read};

pub fn complete() -> Result<(), Error> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).map_err(|e| {
        Error::default()
            .wrap(Oops::StdinReadError)
            .because(e.kind().to_string())
    })?;

    let payload = CompletionPayload {
        model: Model::Gpt4oMini,
        messages: vec![
            Message::new(
                Role::System,
                [
                    "Complete this javascript code.",
                    "Format your response as syntactically correct JavaScript code,",
                    "not markdown."
                ].join(" ")
            ),
            Message::new(
                Role::User,
                input
            ),
        ],
    };
    let response = OpenAI::from_env()?.chat(&payload)?;
    let content = response.choices[0].message.parse()?;
    match content {
        Content::Normal(c) => println!("{}", c),
        Content::Refusal(r) => eprintln!("{}", r),
    };
    Ok(())
}
