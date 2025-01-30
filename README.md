<!-- This README is generated by ./scripts/gen_readme.bash -->

# `yap`

`yap` is a high-level CLI toolkit to help programmers use LLMs for
programming; built in the spirit of Unix single-responsibility programs.

# Features

- [`yap complete`](crate::complete): read a prompt from `STDIN`, print the
  response to `STDOUT`
- [`yap chat [prompt]`](crate::chat): chat with an LLM in your terminal
  - `yap chat --new [prompt]`: begin a chat session in your terminal, with
    persistent chat history via [crate::db]
  - `yap chat --resume [chat-id]`: resume a previous chat from `yap chatlog`
- [`yap annotate`](crate::annotate): receive feedback on chunks of code
- [`yap chatlog`](crate::chatlog): view chat history
- [`yap recap`](crate::recap): view your conversation so far

# Installation

You can compile and install `yap` from source with cargo;

```bash
cargo install --path .
```

To validate the installation, run;

```bash
yap --help
```

# Setup

To start using `yap` you need to set `OPENAI_API_KEY` in your environment.

With an API key available, you can start using `yap`!

# Example Usage

```bash
$ echo "console.log(" | yap complete
  "Hello, World!"
)

$ yap chat How are you doing today\?
I'm just a computer program, so I don't have feelings, but I'm here and
ready to help you with whatever you need! How can I assist you today?

$ yap chat --new "Let's start a new conversation, now"
Sure! What would you like to discuss or work on today?
```

# Additional Documentation

Links below to `[crate::config]`, etc. will be functional if you view the
cargo-docs for this crate;

```bash
cargo doc --open
```

# Configuration

See [crate::config].

# Persistence

See [crate::db].

# Debugging

`yap` uses the [log] and [env_logger] crates. You can configure logging
via the `RUST_LOG` environment variable;

```bash
echo "tell me a story" | RUST_LOG=debug yap complete
```

# Alternatives to `yap`

A brief review of other CLI tool sfor working with LLMs, comparing them
to my goals for `yap`.

<details>
<summary>Comparison to Alternatives</summary>

## [simonw/llm](https://github.com/simonw/llm)

`llm` is basically an abstract interface to LLMs. `llm` is to OpenAI as
kubernetes is to AWS. `llm` offers a CLI and Python library, whereas
`yap` only strives to be a CLI tool and does not expose a library
interface.

Ideally, `yap` is all about helping with programming, using LLMs as a means
to that end. `annotate` is an example of a high-level workflows which use
LLMs, and I plan to add more tools like that to `yap`.

`yap` only supports OpenAI for now, but it should be possible for `yap`
to support many LLM backends in the future, as `llm` does.

## [Aider-AI/aider](https://github.com/Aider-AI/aider)

`aider` is similar to `yap` in the sense that they are both higher-level
tools built on top of LLMs to help with programming. If you like the idea of
an AI REPL which has access to read from your file system, you should check
out `aider`!

`yap` fills a somewhat different role. A lot of `yap` tools fit within the
Unix `STDIN` / `STDOUT` model. It should be very easy, for example, to do
some tricky stuff with `yap` from vim / neovim / emacs, or just from the
shell.

`aider` also heavily drives the version control process, and helps you to
incrementally apply changes to source files, whereas `yap` is happy to
remain orthogonal to version control. I think that this will make `yap`
much simpler to use since `yap` will obviously and directly modify files.
`yap` assumes that you know how to use `git`, so make sure you've checked
in code that is important before letting `yap` go buck-wild in your
codebase!

## [gorilla-llm/gorilla-cli](https://github.com/gorilla-llm/gorilla-cli), [djcopley/ShellOracle](https://github.com/djcopley/ShellOracle?tab=readme-ov-file)

Each of these tools are for help with _using the shell._ I love the shell.
These tools look awesome for getting to know the shell. `yap` isn't meant to
help you use the shell. `yap` is meant to be a tool that exists in your
shell. Right alongside the greats (`cat`, `awk`, `sed`, `grep`, `curl`,
`ssh`, etc.).

## [plandex-ai/plandex](https://github.com/plandex-ai/plandex)

`plandex` most similar to `yap`. `plandex` and `yap` certainly have the same
central motivating thesis - a high-level CLI tool for developing software
with LLMs. A few important differences exist between `plandex` and `yap`;

- `yap` is more of a minimal unix-y tool; it doesn't, for example, concern
  itself with version control or incremental application of changes to source
  files. [Git](https://git-scm.com/) is probably a better tool for version
  control!
- `yap` avoids repl-based workflows, which can be awkward to compose with
  other CLI programs, or integrate into (neo)vim / emacs.
- `yap` has a MIT license, but `plandex` has an aGPL license.
- The `plandex` CLI is a http client which talks to a [proprietary remote server](https://github.com/plandex-ai/plandex/blob/main/app/server/routes.go),
  whereas `yap` is a local-only tool which talks directly to OpenAI or (in
  principle) can run fully offline with local models (though we only support
  OpenAI models for now).

## Other Projects

[`ell`'s](https://github.com/simonmysun/ell) README has a good list of similar
tools besides the ones mentioned here.

</details>
