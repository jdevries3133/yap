//! Error handling for `yap`

#[derive(Debug)]
pub enum Oops {
    OpenAIKeyMissing,
    OpenAIChatResponse,
    OpenAIChatDeserialization,
    OpenAIBadFinishReason,
    OpenAIEmptyChoices,
    OpenAIContentAndRefusal,
    OpenAIEmptyContent,
    StdinReadError,
    XdgConfigError,
    DbError,
    DbNotFound,
    CompletionError,
    ChatError,
    #[allow(unused)]
    Placeholder,
}

impl Oops {
    /// In some cases, there might only be one possible explanation for an
    /// error type, in which case we can centralize those explanations here
    /// instead of needing to use [Error::because] all over the place.
    pub fn explain(&self) -> Option<&'static str> {
        match self {
            Self::OpenAIEmptyChoices => {
                Some("OpenAI did not provide any response choices.")
            }
            Self::OpenAIKeyMissing => {
                Some("set $OPENAI_API_KEY in your environment")
            },
            Self::OpenAIContentAndRefusal => {
                Some("OpenAI message contained `content` and `refusal`. This should never happen.")
            },
            Self::OpenAIEmptyContent => {
                Some("OpenAI messages contains neither `content` nor `refusal`. This should never happen.")
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Oopsie {
    variant: Oops,
    ctx: Option<String>,
}

#[derive(Debug, Default)]
pub struct Error {
    /// A series of unfortunate events, from first to last.
    oopsies: Vec<Oopsie>,
}

/// An adequate and simple error framework. Start by creating an error;
///
/// ```
/// // Start by making a new error.
/// let e = Error::default()
/// // Then, identify what went wrong.
/// e.wrap(Oops::OpenAIKeyMissing);
/// // Optionally, say why.
/// fn bad_stuff() {
///     e.wrap(Oops::OpenAIChatResponse).because(format!(
///         "In function {}, we encountered {}",
///         type_name(bad_stuff),
///         "some other error type"
///     ))
/// }
/// ```
///
/// As errors flow up through a call stack, receivers can call [Self::wrap]
/// and/or [Self::because] to add context to the error.
impl Error {
    /// Append an error-type to the stack.
    pub fn wrap(mut self, oops: Oops) -> Self {
        self.oopsies.push(Oopsie {
            variant: oops,
            ctx: None,
        });
        self
    }
    /// Add a description by mutating the most recent entry on the error stack.
    /// We expect this to be called immediately after a call to [Self::wrap]
    /// to enhance the error-type with details from the context where the
    /// error happened.
    pub fn because(mut self, ctx: String) -> Self {
        if let Some(last) = self.oopsies.last_mut() {
            last.ctx = Some(ctx);
        }
        self
    }
    pub fn display(&self) {
        if self.oopsies.is_empty() {
            return;
        }
        eprintln!("Oops! One or more errors occurred;");
        let alt = "details not available";
        for (indent, item) in self.oopsies.iter().enumerate() {
            let indent = "  ".repeat(indent + 1);
            let er_code = &item.variant;
            let ctx = item.ctx.as_ref();
            if let Some(ctx) = ctx {
                eprintln!("{indent}{er_code:?} :: {ctx}");
            } else if let Some(exp) = er_code.explain() {
                eprintln!("{indent}{er_code:?} :: {exp}");
            } else {
                eprintln!("{indent}{er_code:?} :: {alt}");
            }
        }
    }
}
