mod annotate;
mod chat;
mod complete;
mod err;
mod openai;
mod scaffold;

use clap::{Parser, Subcommand};
use std::{path::PathBuf, process::exit};

#[derive(Debug, Parser)]
#[command(name = "yap", version)]
#[command(about = "Get LLMs to do more than just yap.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[clap(value_enum, default_value_t=openai::Model::Gpt4oMini)]
    #[arg(short, long)]
    model: openai::Model,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print completion for STDIN to STDOUT.
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
    fn dispatch(&self) -> Result<(), err::Error> {
        match self {
            Self::Log { id } => {
                dbg!("log", id);
                Ok(())
            }
            Self::Chat { prompt, resume } => {
                dbg!(prompt, resume);
                chat::chat();
                Ok(())
            }
            Self::Complete => complete::complete(),
            Self::Annotate {
                prompt,
                file,
                line_start,
                line_end,
            } => {
                dbg!(prompt, file, line_start, line_end);
                annotate::annotate();
                Ok(())
            }
            Self::Scaffold {
                template,
                target,
                prompt,
            } => {
                dbg!(template, target, prompt);
                scaffold::scaffold();
                Ok(())
            }
        }
    }
}

fn main() {
    let args: Cli = Cli::parse();
    if let Err(e) = args.command.dispatch() {
        e.display();
        exit(1);
    };
}
