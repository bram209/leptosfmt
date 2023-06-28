use std::path::Path;

use source_file::{format_file_source, FormatError};

mod collect;
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
