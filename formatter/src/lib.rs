use std::path::Path;

use source_file::{format_file_source, FormatError};

mod collect;
mod formatter;
mod source_file;

#[cfg(test)]
mod test_helpers;

pub use formatter::format_macro;
pub use formatter::FormatterSettings;

pub fn format_file(path: &Path, settings: FormatterSettings) -> Result<String, FormatError> {
    let file = std::fs::read_to_string(path)?;
    format_string(file, settings)
}

pub fn format_string(file: String, settings: FormatterSettings) -> Result<String, FormatError> {
    let formatting = format_file_source(&file, settings)?;
    if !settings.allow_changes && file != formatting {
        Err(FormatError::IncorrectFormatError)
    } else {
        Ok(formatting)
    }
}
