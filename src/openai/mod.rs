//! `yap`'s interface to OpenAI

mod chat_api;

use crate::err::{Error, Oops};
use serde::{Deserialize, Serialize};
use std::{default::Default, env};

pub struct OpenAI {
    auth_header: String,
    pub model: Model,
}

impl OpenAI {
    pub fn from_env(preferred_model: Option<Model>) -> Result<Self, Error> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| Error::default().wrap(Oops::OpenAIKeyMissing))?;
        Ok(Self {
            auth_header: format!("Bearer {api_key}"),
            model: preferred_model.unwrap_or_default(),
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

pub use chat_api::{
    chat, CompletionPayload, Content, Message, Model, PayloadOpts,
    ResponseFormat,
};
