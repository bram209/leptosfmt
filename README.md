# leptosfmt

[![crates.io](https://img.shields.io/crates/v/leptosfmt.svg)](https://crates.io/crates/leptosfmt)
[![build](https://img.shields.io/github/actions/workflow/status/bram209/leptosfmt/ci.yml)](https://github.com/bram209/leptosfmt/actions/workflows/ci.yml?query=branch%3Amain)
[![security](https://img.shields.io/github/actions/workflow/status/bram209/leptosfmt/security-audit.yml?label=%F0%9F%9B%A1%EF%B8%8F%20security%20audit)](https://github.com/bram209/leptosfmt/actions/workflows/security-audit.yml?query=branch%3Amain)
[![discord](https://img.shields.io/discord/1031524867910148188?color=%237289DA&label=discord%20%23leptosfmt)](https://discord.gg/YdRAhS7eQB)

A formatter for the leptos view! macro

All notable changes are documented in: [CHANGELOG.md](./CHANGELOG.md)

## Install

`cargo install leptosfmt`

or for trying out unreleased features:

`cargo install --git https://github.com/bram209/leptosfmt.git`

## Usage

```
Usage: leptosfmt [OPTIONS] [INPUT_PATTERNS]...

Arguments:
  [INPUT_PATTERNS]...  A space separated list of file, directory or glob

Options:
  -m, --max-width <MAX_WIDTH>
          Maximum width of each line
  -t, --tab-spaces <TAB_SPACES>
          Number of spaces per tab
  -x, --excludes <EXCLUDE_PATTERNS>
          A space separated list of file, directory or glob
  -c, --config-file <CONFIG_FILE>
          Configuration file
  -s, --stdin
          Format stdin and write to stdout
  -r, --rustfmt
          Format with rustfmt after formatting with leptosfmt (requires stdin)
      --override-macro-names <OVERRIDE_MACRO_NAMES>...
          Override formatted macro names
  -e, --experimental-tailwind
          Format attributes with tailwind
      --tailwind-attr-names <TAILWIND_ATTR_NAMES>...
          Override attributes to be formatted with tailwind [default: class]
  -q, --quiet

      --check
          Check if the file is correctly formatted. Exit with code 1 if not
  -h, --help
          Print help
  -V, --version
          Print version
```

## Using with Rust Analyzer

You have to do two things:
- configure edition in `rustfmt.toml`
- configure RA by setting the `rust-analyzer.rustfmt.overrideCommand` setting

### Configure `rustfmt` edition
**You must** configure `rustfmt` to use the correct edition, place a `rustfmt.toml` file in the root of your project:

```toml
edition = "2021"
# (optional) other config...
```

### Configure RA
<details>
  <summary>Option 1: Using `rust-analyzer.toml` (Recommended)</summary> <br />
  A new way to configure `rust-analyzer` to use `leptosfmt` is to use directory based `rust-analyzer` configuration.
  
  To do this, create a file named `rust-analyzer.toml` in the root of your project with the following content: 
  ```toml
  [rustfmt] 
  overrideCommand = ["leptosfmt", "--stdin", "--rustfmt"]
  # (optional) other config...
  ```
  
  This method of setting up rust-analyzer is editor agnostic to any editor that uses `rust-analyzer` for formatting rust code.
  
  > Note: This feature of `rust-analyzer` is currently unstable and no guarantees are made that this will continue to work across versions. You have to use a recent version of `rust-analyzer` ([2024-06-10](https://github.com/rust-lang/rust-analyzer/releases/tag/2024-06-10) or newer).
</details>

<details>
  <summary>Option 2: Editor specific config</summary> <br />
  
  **VSCode**:
  
  For VSCode users, I recommend to use workpsace settings (CMD + shift + p -> Open workspace settings), so that you can only configure `leptosfmt` for workspaces that are using leptos.
  
  Open your workspace settings and add the following configuration:
  ```json
  {
    "rust-analyzer.rustfmt.overrideCommand": ["leptosfmt", "--stdin", "--rustfmt"]
  }
  ```
  
  **Neovim**:
  
  For Neovim users, I recommend using [neoconf.nvim](https://github.com/folke/neoconf.nvim) for managing project-local LSP configuration, so that you can only configure `leptosfmt` for workspaces that are using leptos.
  
  Alternatively, you may directly configure [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig) by appending the following to your `.setup{}` table:
  ```lua
  lspconfig["rust_analyzer"].setup {
    settings = {
      ["rust-analyzer"] = {
        rustfmt = {
          overrideCommand = { "leptosfmt", "--stdin", "--rustfmt" },
        },
      },
    },
  }
  ```

  **Emacs**:

  For Emacs users, see the relevant [configuration option](https://emacs-lsp.github.io/lsp-mode/page/lsp-rust-analyzer/#lsp-rust-analyzer-rustfmt-override-command) for LSP Mode.
</details>

## Configuration

You can configure all settings through a `leptosfmt.toml` file.

```toml
max_width = 100 # Maximum width of each line
tab_spaces = 4 # Number of spaces per tab
indentation_style = "Auto" # "Tabs", "Spaces" or "Auto"
newline_style = "Auto" # "Unix", "Windows" or "Auto"
attr_value_brace_style = "WhenRequired" # "Always", "AlwaysUnlessLit", "WhenRequired" or "Preserve"
macro_names = [ "leptos::view", "view" ] # Macro names which will be formatted
closing_tag_style = "Preserve" # "Preserve", "SelfClosing" or "NonSelfClosing"

# Attribute values can be formatted by custom formatters
# Every attribute name may only select one formatter (this might change later on)
[attr_values]
class = "Tailwind" # "Tailwind" is the only attribute value formatter available for now
```

To see what each setting does, the see [configuration docs](./docs/configuration.md)

## Examples

**Single file**

Format a specific file by name

`leptosfmt ./examples/counter/src/lib.rs`

**Current directory**

Format all .rs files within the current directory

`leptosfmt .`

**Directory**

Format all .rs files within the examples directory

`leptosfmt ./examples`

**Glob**

Format all .rs files ending with `_test.rs` within the examples directory

`leptosfmt ./examples/**/*_test.rs`

## A note on non-doc comments

Currently this formatter does not support non-doc comments in code blocks. It uses a fork of prettyplease for formatting rust code, and `prettyplease` does not support this. I would like to not diverge this fork too much (so I can easily keep in sync with upstream), therefore I didn't add non-doc comment support in my prettyplease fork for now.
This means that you _can_ use non-doc comments throughout your view macro, as long as they don't reside within code blocks.

> A bit more context: `prettyplease` uses `syn` to parse rust syntax. According to https://doc.rust-lang.org/reference/comments.html#non-doc-comments non-doc comments _are interpreted as a form of whitespace_ by the parser; `syn` basically ignores/skips these comments and does not include them in the syntax tree.

## Pretty-printer algorithm

The pretty-printer is based on Philip Karlton’s Mesa pretty-printer, as described in the appendix to Derek C. Oppen, “Pretty Printing” (1979), Stanford Computer Science Department STAN-CS-79-770, http://i.stanford.edu/pub/cstr/reports/cs/tr/79/770/CS-TR-79-770.pdf.
This algorithm's implementation is taken from `prettyplease` which is adapted from `rustc_ast_pretty`.

The algorithm takes from an input stream of length `n` and an output device with margin width `m`, the algorithm requires time `O(n)` and space `O(m)`.
The algorithm is described in terms of two parallel processes; the first scans the input stream to determine the space required to print logical blocks of tokens; the second uses this information to decide where to break lines of text; the two processes
communicate by means of a buffer of size `o(m)`. The algorithm does not wait for the entire stream to be input, but begins printing as soon as it has received a linefull of input.
