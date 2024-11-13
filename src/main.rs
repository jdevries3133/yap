mod annotate;
mod chat;
mod complete;
mod scaffold;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "yap")]
#[command(about = "Get LLMs to do more than just yap.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Basic context-free completion for STDIN goes to STDOUT.
    Complete,
    /// Chat with LLMs in your terminal.
    Chat {
        /// Use `eval "$(yap chat)"` (without passing a prompt) to start a
        /// new chat.
        prompt: Option<Vec<String>>,
        /// Optional ID of a previous conversation to resume. See also `yap
        /// log`.
        #[arg(short, long)]
        resume: Option<String>,
    },
    /// Ask LLMs to require all or a chunk of a file in response to a prompt.
    Annotate {
        #[arg(short, long)]
        prompt: String,
        #[arg(short, long)]
        file: PathBuf,
        #[arg(long)]
        line_start: Option<u32>,
        #[arg(long)]
        line_end: Option<u32>,
    },
    /// Use LLMs to generate code from a template.
    Scaffold {
        template: PathBuf,
        target: Vec<PathBuf>,
        prompt: String,
    },
    /// View a history of `yap` actions.
    Log {
        /// Pass the ID of a previous action to get more details.
        #[arg(short, long)]
        id: Option<String>,
    },
}

impl Command {
    fn dispatch(&self) {
        match self {
            Self::Log { id } => {
                dbg!("log", id);
            }
            Self::Chat { prompt, resume } => {
                dbg!(prompt, resume);
                chat::chat()
            }
            Self::Complete => complete::complete(),
            Self::Annotate {
                prompt,
                file,
                line_start,
                line_end,
            } => {
                dbg!(prompt, file, line_start, line_end);
                annotate::annotate()
            }
            Self::Scaffold {
                template,
                target,
                prompt,
            } => {
                dbg!(template, target, prompt);
                scaffold::scaffold()
            }
        }
    }
}

fn main() {
    let args: Cli = Cli::parse();
    args.command.dispatch();
}
