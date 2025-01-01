//! <https://platform.openai.com/docs/api-reference/chat>

use super::{OpenAI, Role};
use crate::err::{Error, Oops};
use clap::ValueEnum;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Copy, Clone, ValueEnum, Debug, Serialize)]
pub enum Model {
    #[default]
    #[serde(rename(serialize = "gpt-4o-mini"))]
    Gpt4oMini,
    #[serde(rename(serialize = "gpt-4o"))]
    Gpt4o,
}

#[derive(Debug, Serialize)]
pub struct CompletionPayload {
    pub messages: Vec<Message>,
    pub response_format: ResponseFormat,
    model: Model,
}

#[derive(Default, Debug, Serialize)]
#[serde(tag = "type")]
pub enum ResponseFormat {
    #[default]
    #[serde(rename(serialize = "text"))]
    Text,
    #[serde(rename(serialize = "json_schema"))]
    JsonSchema { json_schema: Value },
}

#[derive(Default)]
pub struct PayloadOpts {
    pub response_format: ResponseFormat,
}

impl CompletionPayload {
    pub fn new(
        open_ai: &OpenAI,
        messages: Vec<Message>,
        opts: PayloadOpts,
    ) -> Self {
        CompletionPayload {
            messages,
            model: open_ai.model,
            response_format: opts.response_format,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Message {
    pub role: Role,
    pub content: Option<String>,
    refusal: Option<String>,
}

pub enum Content<'a> {
    Normal(&'a str),
    Refusal(&'a str),
}

impl Message {
    pub fn new(role: Role, content: String) -> Self {
        Self {
            role,
            content: Some(content),
            refusal: None,
        }
    }
    pub fn parse(&self) -> Result<Content, Error> {
        match (self.content.as_ref(), self.refusal.as_ref()) {
            (Some(_), Some(_)) => {
                Err(Error::default().wrap(Oops::OpenAIContentAndRefusal))
            }
            (Some(content), None) => Ok(Content::Normal(content)),
            (None, Some(refusal)) => Ok(Content::Refusal(refusal)),
            (None, None) => {
                Err(Error::default().wrap(Oops::OpenAIEmptyContent))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CompletionResponse {
    pub choices: Vec<Choice>,
}

impl CompletionResponse {
    pub fn validate(self) -> Result<Self, Error> {
        if self.choices.is_empty() {
            return Err(Error::default().wrap(Oops::OpenAIEmptyChoices));
        };
        if self.choices.iter().all(|Choice { finish_reason, .. }| {
            *finish_reason != FinishReason::Stop
        }) {
            return Err(Error::default()
                .wrap(Oops::OpenAIBadFinishReason)
                .because(format!(
                    r#"Finish reason was "{:?}" instead of "stop""#,
                    self.choices[0].finish_reason
                )));
        };

        Ok(self)
    }
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
    pub finish_reason: FinishReason,
}

#[derive(Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FinishReason {
    Length,
    Stop,
}

pub fn chat(
    open_ai: &OpenAI,
    payload: &CompletionPayload,
) -> Result<CompletionResponse, Error> {
    debug!("Sending chat completion payload: {payload:?}");
    ureq::post("https://api.openai.com/v1/chat/completions")
        .set("Authorization", &open_ai.auth_header)
        .set("Content-Type", "application/json")
        .send_json(payload)
        .map_err(|e| {
            Error::default().wrap_ureq(e).wrap(Oops::OpenAIChatResponse)
        })
        .and_then(|ok| {
            let str = ok.into_string().unwrap();
            serde_json::from_str::<CompletionResponse>(&str).map_err(|e| {
                Error::default()
                    .wrap(Oops::OpenAIChatDeserialization)
                    .because(format!("{e}"))
            })
        })?
        .validate()
}
