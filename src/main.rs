//! # `yap`
//!
//! `yap` is a high-level CLI toolkit to help programmers use LLMs for
//! programming. Built in the spirit of Unix single-responsibility programs.
//!
//! # Features
//!
//! - `yap complete`: read a prompt from `STDIN`, print the response to `STDOUT`
//! - `yap chat [prompt]`: chat with an LLM in your terminal
//!   - `eval "$(yap chat)"`: begin a chat session in your terminal, allowing the
//!     LLM to retain context
//!
//! # Planned Features
//!
//! These planned features are not yet implemented.
//!
//! - `yap annotate`: receive feedback on chunks of code
//! - `yap scaffold`: build smart boilerplate for your own programming patterns
//!
//! # Alternatives to `yap`
//!
//! A brief review of other CLI tool sfor working with LLMs, comparing them
//! to my goals for `yap`.
//!
//! <details>
//! <summary>Comparison to Alternatives</summary>
//!
//! ## [simonw/llm](https://github.com/simonw/llm)
//!
//! `llm` is basically an abstract interface to LLMs. `llm` is to OpenAI as
//! kubernetes is to AWS. `llm` offers a CLI and Python library, whereas
//! `yap` only strives to be a CLI tool and does not expose a library
//! interface.
//!
//! Ideally, `yap` is all about helping with programming, using LLMs as a
//! means to that end. `annotate` and `scaffold` are examples of high-level
//! workflows which use LLMs.
//!
//! `yap` only supports OpenAI for now, but it should be possible for `yap`
//! to support many LLM backends in the future, as `llm` does.
//!
//! ## [Aider-AI/aider](https://github.com/Aider-AI/aider)
//!
//! `aider` is similar to `yap` in the sense that they are both higher-level
//! tools built on top of LLMs to help with programming. If you like the idea of
//! an AI REPL which has access to read from your file system, you should check
//! out `aider`!
//!
//! `yap` fills a somewhat different role. A lot of `yap` tools fit within the
//! Unix `STDIN` / `STDOUT` model. It should be very easy, for example, to do
//! some tricky stuff with `yap` from vim / neovim / emacs, or just from the
//! shell.
//!
//! `aider` also heavily drives the version control process, and helps you to
//! incrementally apply changes to source files, whereas `yap` is happy to
//! remain orthogonal to version control. I think that this will make `yap`
//! much simpler to use since `yap` will obviously and directly modify files.
//! `yap` assumes that you know how to use `git`, so make sure you've checked
//! in code that is important before letting `yap` go buck-wild in your
//! codebase!
//!
//! ## [gorilla-llm/gorilla-cli](https://github.com/gorilla-llm/gorilla-cli), [djcopley/ShellOracle](https://github.com/djcopley/ShellOracle?tab=readme-ov-file)
//!
//! Each of these tools are for help with _using the shell._ I love the shell.
//! These tools look awesome for getting to know the shell. `yap` isn't meant to
//! help you use the shell. `yap` is meant to be a tool that exists in your
//! shell. Right alongside the greats (`cat`, `awk`, `sed`, `grep`, `curl`,
//! `ssh`, etc.).
//!
//! ## [plandex-ai/plandex](https://github.com/plandex-ai/plandex)
//!
//! `plandex` most similar to `yap`. `plandex` and `yap` certainly have the same
//! central motivating thesis - a high-level CLI tool for developing software
//! with LLMs. A few important differences exist between `plandex` and `yap`;
//!
//! - `yap` is more of a minimal unix-y tool; it doesn't, for example, concern
//!   itself with version control or incremental application of changes to source
//!   files. [Git](https://git-scm.com/) is probably a better tool for version
//!   control!
//! - `yap` avoids repl-based workflows, which can be awkward to compose with
//!   other CLI programs, or integrate into (neo)vim / emacs.
//! - `yap` has a MIT license, but `plandex` has an aGPL license.
//! - The `plandex` CLI is a http client which talks to a [proprietary remote server](https://github.com/plandex-ai/plandex/blob/main/app/server/routes.go),
//!   whereas `yap` is a local-only tool which talks directly to OpenAI or (in
//!   principle) can run fully offline with local models (though we only support
//!   OpenAI models for now).
//!
//! ## Other Projects
//!
//! [`ell`'s](https://github.com/simonmysun/ell) README has a good list of similar
//! tools besides the ones mentioned here.
//!
//! </details>
//!
//! # Installation
//!
//! You can compile and install `yap` from source with cargo;
//!
//! ```bash
//! cargo install --path .
//! ```
//!
//! To validate the installation, run;
//!
//! ```bash
//! yap --help
//! ```
//!
//! # Setup
//!
//! To start using `yap` you need to set `OPENAI_API_KEY` in your environment.
//!
//! With an API key available, you can start using `yap`!
//!
//! # Example Usage
//!
//! ```bash
//! $ echo "console.log(" | yap complete
//!   "Hello, World!"
//! )
//!
//! $ yap chat
//! # hint: run `eval "$(yap chat)"` to start a new chat.
//! export YAP_CHAT_HISTORY_FILE='c9f6aa81-a757-4508-8a32-224aaa6e6baa'
//!
//! $ eval "$(yap chat)"
//!
//! $ yap chat How are you doing today\?
//! I'm just a computer program, so I don't have feelings, but I'm here and
//! ready to help you with whatever you need! How can I assist you today?
//! ```
//!
//! # Configuration
//!
//! See [crate::config].
//!
//! # Persistence
//!
//! See [crate::db]. This page also has advanced usage tips for `yap chat`.

mod annotate;
mod chat;
mod complete;
mod config;
mod constants;
mod db;
mod err;
mod openai;
mod scaffold;

use clap::{Parser, Subcommand};
use log::info;
use std::{path::PathBuf, process::exit};

/// `yap`'s command-line interface.
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

/// `yap` subcommands (`complete`, `chat`, etc.)
#[derive(Debug, Subcommand)]
enum Command {
    /// Print completion for STDIN to STDOUT.
    Complete,
    /// Chat with LLMs in your terminal.
    Chat {
        /// Use `eval "$(yap chat)"` (without passing a prompt) to start a
        /// new chat.
        prompt: Option<Vec<String>>,
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
        let open_ai = openai::OpenAI::from_env()?;
        match self {
            Self::Log { id } => {
                info!("logging {id:?}");
                Ok(())
            }
            Self::Chat { prompt } => chat::chat(&open_ai, prompt),
            Self::Complete => complete::complete(&open_ai),
            Self::Annotate {
                prompt,
                file,
                line_start,
                line_end,
            } => {
                info!("annotating prompt = {prompt}, file = {file:?}, start = {line_start:?}, line_end = {line_end:?}");
                annotate::annotate();
                Ok(())
            }
            Self::Scaffold {
                template,
                target,
                prompt,
            } => {
                info!("scaffolding template = {template:?}, target = {target:?}, prompt = {prompt}");
                scaffold::scaffold();
                Ok(())
            }
        }
    }
}

fn main() {
    env_logger::init();
    let args: Cli = Cli::parse();
    if let Err(e) = args.command.dispatch() {
        e.display();
        exit(1);
    };
}
