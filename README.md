# leptosfmt

A formatter for the leptos view! macro

## Install

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
