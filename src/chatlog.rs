use crate::{
    db,
    err::{Error, Oops},
    openai::Role,
    term,
};
use std::fmt::Write;

#[derive(Debug)]
/// A sorted set of conversations, ordered by modified time, descending.
struct ConversationSet(Vec<db::Conversation>);

impl ConversationSet {
    fn new(mut conversations: Vec<db::Conversation>) -> Result<Self, Error> {
        let (result, mut tuples) =
            conversations
                .drain(..)
                .fold((None, Vec::new()), |acc, convo| {
                    let (mut result, mut sorted_vec) = acc;
                    convo
                        .accessed()
                        .map(|time| {
                            sorted_vec.push((time, convo));
                        })
                        .unwrap_or_else(|e| {
                            result = Some(e);
                        });
                    (result, sorted_vec)
                });
        if let Some(err) = result {
            return Err(err);
        };

        tuples.sort_by(|a, b| b.0.cmp(&a.0));
        let sorted_set =
            tuples.drain(..).fold(Vec::new(), |mut acc, (_, convo)| {
                acc.push(convo);
                acc
            });
        // If I want to call it a set I should technically validate that
        // the paths are unique but whatever.
        Ok(Self(sorted_set))
    }

    /// For each message in the conversation set, get the first line of the
    /// most recent message that the user sent.
    fn load(&self, limit: Option<usize>) -> Result<String, Error> {
        let msg_max_len = term::cols() - 3;
        let limit = (limit.unwrap_or(self.0.len()) + 1).min(self.0.len());
        self.0[0..limit]
            .iter()
            .try_fold(String::new(), |mut acc, convo| {
                let convo_id = convo.uuid()?;
                let conversation = db::get_chat(&convo_id)?;
                let message = conversation
                    .iter()
                    .rev()
                    .find(|msg| {
                        matches!(msg.role, Role::User) && msg.content.is_some()
                    })
                    .or(conversation.first())
                    .and_then(|m| m.content.as_ref().map(|c| c.lines().next()))
                    .flatten();
                if let Some(message) = message {
                    write!(acc, "{} :: ", convo.uuid()?).map_err(|e| {
                        Error::default()
                            .wrap(Oops::StringError)
                            .because(format!("failed to write: {e}"))
                    })?;
                    let truncated_msg =
                        &message[0..message.len().min(msg_max_len.into())];
                    acc.push_str(truncated_msg);
                    acc.push_str("...");
                    acc.push('\n');
                }
                Ok(acc)
            })
    }
}

pub fn chatlog(trunc: Option<usize>) -> Result<(), Error> {
    println!(
        "{}",
        ConversationSet::new(db::list_conversations()?)?.load(trunc)?
    );
    println!(
        "To resume a previous chat, run;

    yap chat --resume <uuid>"
    );
    Ok(())
}
