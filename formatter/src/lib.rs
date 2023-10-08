#![deny(clippy::dbg_macro)]

use std::path::Path;

use crop::{Rope, RopeSlice};
use proc_macro2::LineColumn;
pub use source_file::{format_file_source, FormatError};

mod collect;
mod collect_comments;
mod formatter;
mod source_file;
mod view_macro;

#[cfg(test)]
mod test_helpers;

pub use collect::collect_macros_in_file;
pub use formatter::*;

pub fn format_file(path: &Path, settings: FormatterSettings) -> Result<String, FormatError> {
    let file = std::fs::read_to_string(path)?;
    format_file_source(&file, settings)
}

fn get_text_beween_spans(rope: &Rope, start: LineColumn, end: LineColumn) -> RopeSlice<'_> {
    let start_byte = line_column_to_byte(rope, start);
    let end_byte = line_column_to_byte(rope, end);

    return rope.byte_slice(start_byte..end_byte);
}
fn line_column_to_byte(source: &Rope, point: proc_macro2::LineColumn) -> usize {
    let line_byte = source.byte_of_line(point.line - 1);
    let line = source.line(point.line - 1);
    let char_byte: usize = line.chars().take(point.column).map(|c| c.len_utf8()).sum();
    line_byte + char_byte
}
