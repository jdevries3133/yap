use crate::err::{Error, Oops};
use std::process::Command;

const DEFAULT_COLS: u16 = 80;

#[cfg(target_os = "windows")]
pub fn cols() -> u16 {
    80
}

#[cfg(not(target_os = "windows"))]
pub fn cols() -> u16 {
    Command::new("tput")
        .args(["cols"])
        .output()
        .map_err(|e| {
            Error::default()
                .wrap(Oops::CommandError)
                .because(format!("tput command failed: {e}"))
        })
        .and_then(|output| {
            String::from_utf8(output.stdout).map_err(|e| {
                Error::default()
                    .wrap(Oops::StringError)
                    .because(format!("could not parse tput output: {e}"))
            })
        })
        .and_then(|s| {
            s.trim().parse::<u16>().map_err(|e| {
                Error::default().wrap(Oops::StringError).because(format!(
                    r#"could not convert string "{s}" into a u16: {e}"#
                ))
            })
        })
        .unwrap_or_else(|e| {
            log::error!("{e}");
            DEFAULT_COLS
        })
}
