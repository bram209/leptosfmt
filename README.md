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
Usage: leptosfmt [OPTIONS] <INPUT_PATTERN>

Arguments:
  <INPUT_PATTERN>  A file, directory or glob

Options:
  -m, --max-width <MAX_WIDTH>    
  -t, --tab-spaces <TAB_SPACES>  
  -c, --config_file <CONFIG_FILE>          
  -h, --help                     Print help
  -V, --version                  Print version
```

## Known issues

- **End of line** non-doc comments (with two forward slashes) are not yet supported.

## Configuration
You can configure all settings through a `leptosfmt.toml` file.

```toml
max_width = 100
tab_spaces = 4
attr_value_brace_style = "WhenRequired" # "Always", "AlwaysUnlessLit", "WhenRequired" or "Preserve"

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

## Pretty-printer algorithm

The pretty-printer is based on Philip Karlton’s Mesa pretty-printer, as described in the appendix to Derek C. Oppen, “Pretty Printing” (1979), Stanford Computer Science Department STAN-CS-79-770, http://i.stanford.edu/pub/cstr/reports/cs/tr/79/770/CS-TR-79-770.pdf.
This algorithm's implementation is taken from `prettyplease` which is adapted from `rustc_ast_pretty`.

The algorithm takes from an input stream of length `n` and an output device with margin width `m`, the algorithm requires time `O(n)` and space `O(m)`.
The algorithm is described in terms of two parallel processes; the first scans the input stream to determine the space required to print logical blocks of tokens; the second uses this information to decide where to break lines of text; the two processes
communicate by means of a buffer of size `o(m)`. The algorithm does not wait for the entire stream to be input, but begins printing as soon as it has received a linefull of input.
