//! `yap`'s interface to OpenAI

mod assistants_api;
mod chat_api;

use crate::err::{Error, Oops};
pub use chat_api::{chat, CompletionPayload, Content, Message, Model};
use serde::{Deserialize, Serialize};
use std::env;

pub struct OpenAI {
    auth_header: String,
}

impl OpenAI {
    pub fn from_env() -> Result<Self, Error> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| Error::default().wrap(Oops::OpenAIKeyMissing))?;
        Ok(Self {
            auth_header: format!("Bearer {api_key}"),
        })
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    #[default]
    User,
    Assistant,
}
