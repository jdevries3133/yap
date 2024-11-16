//! Load your yap config from the `$XDG_CONFIG_HOME/yap` directory.

use crate::{
    constants,
    err::{Error, Oops},
};
use log::{debug, log_enabled, Level::Debug};
use std::{
    env::{self, VarError},
    fs::{create_dir_all, read_to_string},
    path::PathBuf,
};

/// Get the yap configuration directory.
///
/// Returns [Option::None] if the dir does not exist.
///
/// Returns errors if `$XDG_CONFIG_HOME` is messed up.
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

/// The system prompt for `yap complete` is read from `complete_prompt.txt`.
pub fn get_system_prompt_for_completion() -> Result<String, Error> {
    let dir = get_or_create_yap_cfg_dir().map_err(|e| {
        e.wrap(Oops::XdgConfigError)
            .because("Error while getting system prompt for completion".into())
    })?;
    let prompt_path = dir.join("complete_prompt.txt");
    if !prompt_path.exists() {
        debug!("Using default system prompt");
        return Ok(constants::DEFAULT_COMPLETION_PROMPT.into());
    }
    let prompt = read_to_string(&prompt_path).map_err(|e| {
        Error::default().wrap(Oops::XdgConfigError).because(format!(
            "Could not create {} due to an OS error: {:?}",
            prompt_path.to_string_lossy(),
            e
        ))
    })?;
    if log_enabled!(Debug) {
        let mut p = prompt.clone();
        p.truncate(50);
        debug!("Using system prompt {p}...");
    };

    Ok(prompt)
}
