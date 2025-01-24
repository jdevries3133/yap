use crate::{
    db,
    err::{Error, Oops},
};

pub fn recap() -> Result<(), Error> {
    let active_chat_id = db::get_active_chat()?.map_or_else(
        || Err(Error::default().wrap(Oops::RecapError).because(
            "Cannot recap; no chat is active! Hint: run `yap chat [prompt]` to get a new conversation started".to_string()
        )), Ok)?;
    let conversation_content = db::get_chat(&active_chat_id)?;
    let convo = conversation_content
        .iter()
        .fold(Vec::new(), |mut acc, msg| {
            if let Some(c) = &msg.content {
                let mut prefixed_str = format!("[{}]: {}", msg.role, c);
                if prefixed_str.ends_with('\n') {
                    prefixed_str.push('\n');
                }
                acc.push(prefixed_str)
            }
            acc
        })
        .join("===\n");
    print!("{}", convo);
    Ok(())
}
