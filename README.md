# `yap`: the CLI to LLMs

`yap` is a high-level CLI toolkit to help programmers use LLMs for programming.
Built in the spirit of Unix single-responsibility programs.

# Features

- `yap complete`: read a prompt from `STDIN`, print the response to `STDOUT`
- `yap chat [prompt]`: chat with an LLM in your terminal
  - `eval "$(yap chat)"`: begin a chat session in your terminal, allowing the
    LLM to retain context by setting the `YAP_SESSION_ID` environment variable.
    This maps onto backend features like [OpenAI Assistants /
    Threads](https://platform.openai.com/docs/api-reference/assistants)
  - `eval "$(yap chat --resume [session_id])"`
- `yap annotate`: receive feedback on chunks of code
- `yap scaffold`: build smart boilerplate for your own programming patterns

# Comparison to Alternatives

## [simonw/llm](https://github.com/simonw/llm)

`llm` is basically an abstract interface to LLMs. `yap`, on the other hand,
tries to ship a toolkit built _on top_ of LLMs, which is hopefully useful for
developing software, and other CLI activities (writing email, note-taking, data
bunging).

`llm` is a CLI and a Python library, but exposing a library is a non-goal of
`yap`.

`yap` is a less mature project than `llm`, and it supports fewer LLMs.

## [Aider-AI/aider](https://github.com/Aider-AI/aider)

`aider` is similar to `yap` in the sense that they are both higher-level tools
built on top of LLMs to help with programming. If you like the idea of an AI
REPL which has access to read from your file system, you should check out
`aider`!

`yap` fills a somewhat different role. A lot of `yap` tools fit within the Unix
`STDIN` / `STDOUT` model. It should be very easy, for example, to do some tricky
stuff with `yap` from vim / neovim / emacs, or just from the shell.

## [gorilla-llm/gorilla-cli](https://github.com/gorilla-llm/gorilla-cli), [djcopley/ShellOracle](https://github.com/djcopley/ShellOracle?tab=readme-ov-file)

Each of these tools are for help with _using the shell._ I love the shell. These
tools look awesome for getting to know the shell. `yap` isn't meant to help you
use the shell. `yap` is meant to be a tool that exists in your shell. Right
alongside the greats (`cat`, `awk`, `sed`, `grep`, `curl`, `ssh`, etc.). 

## [plandex-ai/plandex](plandex-ai/plandex)

`plandex` most similar to `yap`. `plandex` and `yap` certainly have the same
central motivating thesis - a high-level CLI tool for developing software with
LLMs. A few important differences exist between `plandex` and `yap`;

- `yap` is more of a minimal unix-y tool; it doesn't, for example, concern
  itself with version control or incremental application of changes to source
  files. [Git](https://git-scm.com/) is probably a better tool for version
  control!
- `yap` avoids repl-based workflows, which can be awkward to compose with other
  CLI programs, or integrate into (neo)vim / emacs.
- `yap` has a MIT license, but `plandex` has an aGPL license.
- The `plandex` CLI is a http client which talks to a [remote
  server](https://github.com/plandex-ai/plandex/blob/main/app/server/routes.go),
  whereas `yap` is a local-only tool

## Other Projects

[`ell`'s](https://github.com/simonmysun/ell) README has a good list of similar
tools besides the ones mentioned here.
