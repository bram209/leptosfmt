use std::path::Path;

use source_file::{format_file_source, FormatError};

mod collect;
mod formatter;
mod source_file;

#[cfg(test)]
mod test_helpers;

pub use collect::collect_macros_in_file;
pub use formatter::*;

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

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn no_allow_changes_incorrect_formatting() {
        let source = indoc! {r#"
            fn main() {
                view! {   cx ,  <div>  <span>"hello"</span></div>  };
            }
        "#}
        .to_string();

        let result = format_string(
            source,
            FormatterSettings {
                allow_changes: false,
                ..Default::default()
            },
        );

        match result {
            Err(FormatError::IncorrectFormatError) => {}
            Ok(_) => panic!("expected result to be an err"),
            Err(_) => panic!("expected result to be of the IncorrectFormatError variant"),
        }
    }

    #[test]
    fn no_allow_changes_correct_formatting() {
        let source = indoc! {r#"
        fn main() {
            view! { cx,
                <div>
                    <span>"hello"</span>
                </div>
            };
        }
        "#}
        .to_string();

        let result = format_string(
            source,
            FormatterSettings {
                allow_changes: false,
                ..Default::default()
            },
        );

        assert!(result.is_ok());
    }
}
