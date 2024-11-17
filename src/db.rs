//! `yap` persists data into `$HOME/.local/state/yap`
//!
//! To clear all `yap` chat history, run `rm -rf $HOME/.local/state/yap`.
//!
//! You can hack with `yap` a bit, because the environment variable
//! `YAP_CHAT_HISTORY_FILE` is only ever used to identify a JSON file where the
//! chat history is stored;
//!
//! ```bash
//! cat ~/.local/state/yap/chats/$YAP_CHAT_HISTORY_FILE.json | jq
//! ```
//!
//! You can set `YAP_CHAT_HISTORY_FILE` to whatever you like to create named
//! chats. I.e,
//!
//! ```bash
//! export YAP_CHAT_HISTORY_FILE=my-chat
//! yap chat hello again, my old friend
//! ```

use crate::{
    err::{Error, Oops},
    openai::Message,
};
use std::{
    env,
    fs::{create_dir_all, File},
    path::PathBuf,
};
use uuid::Uuid;

fn get_or_create_persistence_dir() -> Result<PathBuf, Error> {
    let dir = env::var("HOME")
        .map_err(|e| match e {
            env::VarError::NotPresent => Error::default()
                .wrap(Oops::DbError)
                .because("$HOME is not present in the environment".into()),
            env::VarError::NotUnicode(_) => Error::default()
                .wrap(Oops::DbError)
                .because("$HOME is not a unicode string".into()),
        })
        .map(PathBuf::from)?
        .join(".local")
        .join("state")
        .join("yap");
    if !dir.exists() {
        create_dir_all(&dir).map_err(|e| {
            Error::default().wrap(Oops::DbError).because(format!(
                "Failed to create ~/.local/state/yap directory: {e}"
            ))
        })?;
    }
    Ok(dir)
}

fn get_or_create_chat_directory() -> Result<PathBuf, Error> {
    let dir = get_or_create_persistence_dir()?;
    let chat_file_dir = dir.join("chats");
    if !chat_file_dir.exists() {
        create_dir_all(&chat_file_dir).map_err(|e| {
            Error::default()
                .wrap(Oops::DbError)
                .because(format!("Failed to create chat subdirectory: {e}"))
        })?;
    }
    Ok(chat_file_dir)
}

pub fn get_chat(id: &Uuid) -> Result<Vec<Message>, Error> {
    let chat_file_dir = get_or_create_chat_directory().map_err(|e| {
        e.wrap(Oops::DbError).because("during `get_chat`".into())
    })?;
    let chat_file_path = chat_file_dir.join(format!("{id}.json"));

    if !chat_file_path.exists() {
        return Ok(vec![]);
    }

    let chat_file = File::open(&chat_file_path).map_err(|e| {
        Error::default().wrap(Oops::DbNotFound).because(format!(
            "Could not open chat file at {:?}: {e}",
            chat_file_dir
        ))
    })?;

    let messages: Vec<Message> =
        serde_json::from_reader(chat_file).map_err(|e| {
            Error::default().wrap(Oops::DbError).because(format!(
                "Failed to deserialize chat file at {:?}: {e}",
                chat_file_dir
            ))
        })?;

    Ok(messages)
}

pub fn save_chat(id: &Uuid, messages: &[Message]) -> Result<(), Error> {
    let chat_file_path = get_or_create_chat_directory()
        .map_err(|e| {
            e.wrap(Oops::DbError).because("during `save_chat`".into())
        })?
        .join(format!("{id}.json"));

    let chat_file = File::create(&chat_file_path).map_err(|e| {
        Error::default().wrap(Oops::DbError).because(format!(
            "Could not open or create chat file at {:?}: {e}",
            chat_file_path
        ))
    })?;

    serde_json::to_writer(chat_file, &messages).map_err(|e| {
        Error::default().wrap(Oops::DbError).because(format!(
            "Failed to serialize chat to file at {:?}: {e}",
            chat_file_path
        ))
    })?;

    Ok(())
}
