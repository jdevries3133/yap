//! `yap` persists data into `$HOME/.local/state/yap`

use crate::{
    err::{Error, Oops},
    openai::Message,
};
use log::debug;
use std::{
    env,
    fs::{create_dir_all, File, Metadata},
    path::PathBuf,
    time::SystemTime,
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

#[derive(Debug)]
pub struct Conversation {
    metadata: Metadata,
    pub path: PathBuf,
}

impl Conversation {
    pub fn accessed(&self) -> Result<SystemTime, Error> {
        self.metadata.accessed()
            .map_err(|e| Error::default().wrap(Oops::OsError).because(format!(
                "Could not get last accessed time from file metadata related to {:?}: {}",
                self.path,
                e
            )))
    }
    pub fn uuid(&self) -> Result<Uuid, Error> {
        parse_uuid(&self.path)
    }
}

fn parse_uuid(path: &PathBuf) -> Result<Uuid, Error> {
    let name = path
        .file_name()
        .ok_or(Error::default().wrap(Oops::DbError).because(format!(
            "conversation path has no filename ({:?})",
            path
        )))?
        .to_str()
        .ok_or(Error::default().wrap(Oops::DbError).because(format!(
            "for path {:?}, cannot convert filename into string",
            path
        )))?;
    let mut parts = name.split(".");
    let uuid_str = parts.next().ok_or(
        Error::default()
            .wrap(Oops::DbError)
            .because(format!("cannot find first part of {name}")),
    )?;
    let extension = parts.next().ok_or(
        Error::default()
            .wrap(Oops::DbError)
            .because(format!("cannot find second part of {name}",)),
    )?;
    if extension != "json" {
        return Err(Error::default().wrap(Oops::DbError).because(format!(
            "file extension {} != json; for file {:?}",
            extension, path
        )));
    };
    if parts.enumerate().fold(0, |_, (i, _)| i) != 0 {
        return Err(Error::default().wrap(Oops::DbError).because(format!(
            "file name has more parts than expected: {name}"
        )));
    };
    Uuid::parse_str(uuid_str).map_err(|e| {
        Error::default()
            .wrap(Oops::DbError)
            .because(format!("cannot parse UUID from file {path:?}: {e}"))
    })
}

pub fn list_conversations() -> Result<Vec<Conversation>, Error> {
    get_or_create_chat_directory().map_err(|e| {
        e.wrap(Oops::DbError).because("during `list_conversations`: {e}".into())
    })?
    .read_dir()
        .map_err(|e| {
            Error::default()
                .wrap(Oops::DbError)
                .because(format!("could not read chat dir: {e}"))
        })
        .map(|files| {
            #[allow(clippy::manual_try_fold)]
            files.fold(Ok(Vec::new()), |acc, file| {
                match (acc, file) {
                    (Ok(mut convos), Ok(file)) => {
                            file.metadata()
                                .map_err(|e|
                                    Error::default()
                                        .wrap(Oops::DbError)
                                        .because(
                                            format!(
                                                "could not read metadata for file {file:?}: {e}"
                                            )
                                        )
                                )
                            .map(|metadata| {
                                convos.push(Conversation {
                                    metadata,
                                    path: file.path()
                                });
                            })?;
                        Ok(convos)
                    },
                    (_, Err(e)) => {
                        Err(
                            Error::default()
                                .wrap(Oops::DbError)
                                .because(
                                    format!(
                                        "read_dir error encountered: {e}"
                                    )
                                )
                        )
                    },
                    (Err(e), _) => Err(e)
                }
            })
        })?
}

fn get_active_chat_path() -> Result<PathBuf, Error> {
    let dir = get_or_create_persistence_dir()?;
    Ok(dir.join("active_chat"))
}

pub fn get_active_chat() -> Result<Option<Uuid>, Error> {
    let active_chat_path = get_active_chat_path()?;
    if !active_chat_path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(&active_chat_path).map_err(|e| {
        Error::default().wrap(Oops::DbError).because(format!(
            "could not read active chat: {active_chat_path:?}: {e}"
        ))
    })?;
    Ok(Some(Uuid::parse_str(&contents).map_err(|e| {
        debug!("found bad file contents: {contents}");
        Error::default()
            .wrap(Oops::DbError)
            .because(format!("active chat is not a uuid ({e})"))
    })?))
}

pub fn set_chat_id(uuid: &Uuid) -> Result<(), Error> {
    let active_chat_path = get_active_chat_path()?;
    std::fs::write(&active_chat_path, uuid.to_string()).map_err(|e| {
        Error::default()
            .wrap(Oops::DbError)
            .because(format!(
                    "could not write new chat ID {uuid} to chat path {active_chat_path:?}: {e}"
            ))
    })?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_uuid() {
        let uuid =
            Uuid::parse_str("4a016e25-60f4-4355-8165-97abff7be79b").unwrap();
        let path =
            PathBuf::from(format!("/home/foo/.local/state/yap/{}.json", uuid));
        let result = parse_uuid(&path).unwrap();
        assert_eq!(result, uuid);
    }
}
