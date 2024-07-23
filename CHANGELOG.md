# Changelog

All notable changes to this project will be documented in this file.

## [0.1.30] - 2024-07-23

### Bug Fixes

- Inline unquoted text + expr block combinations ([#110](https://github.com/bram209/leptosfmt/issues/110))
- Trailing whitespace issue when using codeblocks ([#130](https://github.com/bram209/leptosfmt/issues/130))

### Continuous Integration

- Manually trigger changelog workflow

### Documentation

- Directory-based rust-analyzer configuration ([#127](https://github.com/bram209/leptosfmt/issues/127))
- Fix default attr_value_brace_style

### Features

- Add option to override macro name ([#109](https://github.com/bram209/leptosfmt/issues/109))
- Preserve formatting of unknown macros ([#121](https://github.com/bram209/leptosfmt/issues/121))
- Add tailwind support for attr values ([#122](https://github.com/bram209/leptosfmt/issues/122))
- Support elements with generics ([#128](https://github.com/bram209/leptosfmt/issues/128))
- Customisable self-closing tag behaviour on non-void elements ([#123](https://github.com/bram209/leptosfmt/issues/123))

### Miscellaneous Tasks

- Update prettyplease and add as git submodule ([#119](https://github.com/bram209/leptosfmt/issues/119))

## [0.1.18] - 2023-12-22

### Bug Fixes

- Don't print new line to stdout ([#95](https://github.com/bram209/leptosfmt/issues/95))

### Features

- Add indentation style (tabs/spaces) & newline style (LF/CRLF) settings ([#90](https://github.com/bram209/leptosfmt/issues/90))

### Miscellaneous Tasks

- Rename printer setting `spaces` to `tab_spaces` for consistency

## [0.1.17] - 2023-10-10

### Bug Fixes

- Ignore rustfmt output on error status ([#89](https://github.com/bram209/leptosfmt/issues/89))

### Features

- Call rustfmt after formatting with leptosfmt (requires stdin) ([#88](https://github.com/bram209/leptosfmt/issues/88))

## [0.1.16] - 2023-10-09

### Bug Fixes

- Retain unicode & raw string formatting ([#87](https://github.com/bram209/leptosfmt/issues/87))

## [0.1.15] - 2023-10-08

### Bug Fixes

- Workaround bug with proc_macro2 regarding multibyte chars ([#85](https://github.com/bram209/leptosfmt/issues/85))

### Features

- Automatically detect CRLF or LF line endings ([#81](https://github.com/bram209/leptosfmt/issues/81))

## [0.1.14] - 2023-09-06

### Bug Fixes

- Formatting comments that include '//' ([#68](https://github.com/bram209/leptosfmt/issues/68))
- Dont touch file when there are no formatting changes ([#69](https://github.com/bram209/leptosfmt/issues/69))
- Softbreak when elem has single raw text child node ([#73](https://github.com/bram209/leptosfmt/issues/73))

### Documentation

- Add clarification about the usage of non-doc comments ([#70](https://github.com/bram209/leptosfmt/issues/70))

### Features

- Add check mode ([#72](https://github.com/bram209/leptosfmt/issues/72))

### Miscellaneous Tasks

- Remove dbg! statement and enable clippy dbg! lint

## [0.1.12] - 2023-07-31

### Bug Fixes

- Rework non-doc comments ([#48](https://github.com/bram209/leptosfmt/issues/48))
- Implement workaround for non-doc comments ([#49](https://github.com/bram209/leptosfmt/issues/49))
- Comment extracting in attribute with block expr
- View macro indentation issues ([#59](https://github.com/bram209/leptosfmt/issues/59))

### Documentation

- Fix typo in `"AlwaysUnlessLit"` example ([#41](https://github.com/bram209/leptosfmt/issues/41))

### Features

- Add stdin and quiet mode ([#30](https://github.com/bram209/leptosfmt/issues/30))
- Rework non-doc comments v2 ([#52](https://github.com/bram209/leptosfmt/issues/52))
- Support leptos 0.5+ ([#53](https://github.com/bram209/leptosfmt/issues/53))

### Miscellaneous Tasks

- Remove dbg! (Closes: #55) ([#56](https://github.com/bram209/leptosfmt/issues/56))

## [0.1.9] - 2023-06-29

### Bug Fixes

- Ignore comments outside view macro ([#40](https://github.com/bram209/leptosfmt/issues/40))

## [0.1.8] - 2023-06-29

### Bug Fixes

- Don't emit empty line when multiline opening tag ([#38](https://github.com/bram209/leptosfmt/issues/38))

## [0.1.7] - 2023-06-29

### Bug Fixes

- Respect string whitespace ([#37](https://github.com/bram209/leptosfmt/issues/37))

### Continuous Integration

- Build binary for MacOS arm64

### Features

- Respect single empty line ([#36](https://github.com/bram209/leptosfmt/issues/36))

## [0.1.6] - 2023-06-29

### Features

- Migrate to rstml ([#32](https://github.com/bram209/leptosfmt/issues/32))
- Non-doc comments within rsx ([#4](https://github.com/bram209/leptosfmt/issues/4))

### Miscellaneous Tasks

- Share printer implementation with leptosfmt-prettyplease ([#31](https://github.com/bram209/leptosfmt/issues/31))

## [0.1.5] - 2023-05-29

### Bug Fixes

- Format view! macro with global class ([#21](https://github.com/bram209/leptosfmt/issues/21))
- View! macro bytecode range when contains unicode characters larger than 1 byte ([#22](https://github.com/bram209/leptosfmt/issues/22))
- Improve identation of view! macro ([#23](https://github.com/bram209/leptosfmt/issues/23))

### Features

- Read settings from a config file ([#25](https://github.com/bram209/leptosfmt/issues/25))

### Miscellaneous Tasks

- Make macro collecting functionality public
- Export token types ([#24](https://github.com/bram209/leptosfmt/issues/24))

## [0.1.4] - 2023-03-27

### Bug Fixes

- Multiline strings ([#8](https://github.com/bram209/leptosfmt/issues/8))

### Continuous Integration

- Fix changelog generation config
- Only trigger CI when .rs file changed

### Features

- Attribute value brace style ([#10](https://github.com/bram209/leptosfmt/issues/10))

## [0.1.3] - 2023-03-23

### Bug Fixes

- Formatting for html comments ([#1](https://github.com/bram209/leptosfmt/issues/1))
- Formatting doctype ([#2](https://github.com/bram209/leptosfmt/issues/2))

### Continuous Integration

- Automate publish process ([#6](https://github.com/bram209/leptosfmt/issues/6))
- Automatically update changelog
- Update comitter name for automatic changelog commit

### Features

- Changelog generation ([#5](https://github.com/bram209/leptosfmt/issues/5))

### Miscellaneous Tasks

- Add CI group to changelog
- Prepare for 0.1.3
- Set repository in printer Cargo.toml

<!-- generated by git-cliff -->
