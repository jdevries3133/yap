use crate::err::{Error, Oops};
use serde::{Deserialize, Serialize};
use std::env;

pub struct OpenAI {
    auth_header: String,
}

#[derive(Debug, Serialize)]
pub enum Model {
    #[serde(rename(serialize = "gpt-4o-mini"))]
    Gpt40Mini,
}

#[derive(Debug, Serialize)]
pub struct CompletionPayload {
    pub messages: Vec<Message>,
    pub model: Model,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Message {
    pub role: Role,
    content: Option<String>,
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

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    #[default]
    User,
    Assistant,
}

#[derive(Debug, Deserialize)]
pub struct CompletionResponse {
    pub choices: Vec<Choice>,
}

impl CompletionResponse {
    fn validate(self) -> Result<Self, Error> {
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

impl OpenAI {
    pub fn from_env() -> Result<Self, Error> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| Error::default().wrap(Oops::OpenAIKeyMissing))?;
        Ok(Self {
            auth_header: format!("Bearer {api_key}"),
        })
    }
    pub fn chat(
        &self,
        payload: &CompletionPayload,
    ) -> Result<CompletionResponse, Error> {
        ureq::post("https://api.openai.com/v1/chat/completions")
            .set("Authorization", &self.auth_header)
            .set("Content-Type", "application/json")
            .send_json(payload)
            .map_err(|e| {
                Error::default()
                    .wrap(Oops::OpenAIChatResponse)
                    .because(format!("{e}"))
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
}
