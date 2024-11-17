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

# Planned Features

- `yap annotate`: receive feedback on chunks of code
- `yap scaffold`: build smart boilerplate for your own programming patterns

# More Info

Run `cargo doc`, and then open up `doc/yap/index.html` in your browser to learn
more!
