# leptosfmt

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
  -m, --max-width <MAX_WIDTH>    [default: 100]
  -t, --tab-spaces <TAB_SPACES>  [default: 4]
  -h, --help                     Print help
  -V, --version                  Print version
```

## Examples

**Single file**

Format all .rs files within the current directory

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
