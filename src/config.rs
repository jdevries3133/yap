//! Yap configuration files are loaded from the `$XDG_CONFIG_HOME/yap`
//! directory. To figure out exactly where this is on your system, try;
//!
//! ```bash
//! echo "Put yap configs in this folder: $XDG_CONFIG_HOME/yap"
//! ```
//!
//! Configuration files supported by `yap` are as follows;
//!
//! - `chat_system_prompt.txt`: specify the system prompt provided to the LLM at
//!   the start of each chat. This prompt is used for any new chats.
//! - `complete_system_prompt.txt`: specify the system prompt for `yap
//!   complete`. This prompt is sent with every invocation of `yap complete`.
//! - `annotate_system_prompt.txt`: specify the system prompt for `yap
//!   annotate`. This prompt is sent with every invocation of `yap annotate`.

use crate::err::{Error, Oops};
use log::debug;
use std::{
    env::{self, VarError},
    fs::{create_dir_all, read_to_string},
    path::PathBuf,
};

/// Get the yap configuration directory. Recursively creates the directory
/// via [create_dir_all] if it does not exist.
///
/// Returns errors if `$XDG_CONFIG_HOME` is missing or not unicode.
fn get_or_create_yap_cfg_dir() -> Result<Box<PathBuf>, Error> {
    let dir = env::var("XDG_CONFIG_HOME").map_err(|e| match e {
        VarError::NotUnicode(_) => Error::default()
            .wrap(Oops::XdgConfigError)
            .because("$XDG_CONFIG_HOME is not a unicode string".into()),
        VarError::NotPresent => Error::default()
            .wrap(Oops::XdgConfigError)
            .because("$XDG_CONFIG_HOME is not defined.".into()),
    })?;
    let dir = PathBuf::from(dir).join("yap");
    if dir.exists() {
        Ok(Box::new(dir))
    } else {
        create_dir_all(&dir).map_err(|e| {
            Error::default().wrap(Oops::XdgConfigError).because(format!(
                "OS error while creating {}/yap: {:?}",
                dir.to_string_lossy(),
                e
            ))
        })?;
        Ok(Box::new(dir))
    }
}

#[allow(clippy::enum_variant_names)]
pub enum ConfigFile {
    CompleteSystemPrompt,
    ChatSystemPrompt,
    AnnotateSystemPrompt,
}

impl ConfigFile {
    fn filename(&self) -> &'static str {
        match self {
            Self::ChatSystemPrompt => "chat_system_prompt.txt",
            Self::CompleteSystemPrompt => "complete_system_prompt.txt",
            Self::AnnotateSystemPrompt => "annotate_system_prompt.txt",
        }
    }
    pub fn load(&self) -> Result<Option<String>, Error> {
        let dir = get_or_create_yap_cfg_dir().map_err(|e| {
            e.wrap(Oops::XdgConfigError).because(
                "Error while getting system prompt for completion".into(),
            )
        })?;
        let prompt_path = dir.join(self.filename());
        if !prompt_path.exists() {
            debug!("config file {} does not exist", self.filename());
            return Ok(None);
        }
        let prompt = read_to_string(&prompt_path).map_err(|e| {
            Error::default().wrap(Oops::XdgConfigError).because(format!(
                "Could not read_to_string({}) due to an OS error: {:?}",
                prompt_path.to_string_lossy(),
                e
            ))
        })?;

        debug!("Loaded config file {}", self.filename());

        Ok(Some(prompt))
    }
}
