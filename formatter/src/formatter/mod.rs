use std::collections::HashMap;

use leptosfmt_pretty_printer::{Printer, PrinterSettings};

mod attribute;
mod element;
mod expr;
mod fragment;
mod mac;
mod node;

pub use mac::format_macro;
pub use mac::ViewMacro;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum AttributeValueBraceStyle {
    Always,
    AlwaysUnlessLit,
    WhenRequired,
    Preserve,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct FormatterSettings {
    // Maximum width of each line
    pub max_width: usize,

    // Number of spaces per tab
    pub tab_spaces: usize,

    // Determines placement of braces around single expression attribute values
    pub attr_value_brace_style: AttributeValueBraceStyle,
}

impl Default for FormatterSettings {
    fn default() -> Self {
        Self {
            max_width: 100,
            tab_spaces: 4,
            attr_value_brace_style: AttributeValueBraceStyle::WhenRequired,
        }
    }
}

impl From<&FormatterSettings> for PrinterSettings {
    fn from(value: &FormatterSettings) -> Self {
        Self {
            margin: value.max_width as isize,
            indent: value.tab_spaces as isize,
            min_space: 60,
        }
    }
}

pub struct Formatter<'a> {
    pub printer: &'a mut leptosfmt_pretty_printer::Printer,
    pub settings: FormatterSettings,
    last_line_check: Option<usize>,
    comments: HashMap<usize, &'a str>,
}

impl<'a> Formatter<'a> {
    pub fn new(
        settings: FormatterSettings,
        printer: &'a mut Printer,
        comments: HashMap<usize, &'a str>,
    ) -> Self {
        Self {
            printer,
            settings,
            comments,
            last_line_check: None,
        }
    }

    pub fn with_source(
        settings: FormatterSettings,
        printer: &'a mut Printer,
        source: &'a str,
    ) -> Self {
        Self {
            printer,
            settings,
            comments: source
                .lines()
                .enumerate()
                .filter_map(|(i, l)| l.split("//").nth(1).map(|l| (i, l)))
                .collect(),
            last_line_check: None,
        }
    }

    pub fn write_comments(&mut self, line_index: usize) {
        let last = self.last_line_check.unwrap_or(0);
        let line_diff = self
            .last_line_check
            .map(|last| line_index - last)
            .unwrap_or(0);

        self.last_line_check = Some(line_index);

        let comments: Vec<_> = (last..=line_index)
            .filter_map(|l| self.comments.remove(&l))
            .collect();

        let empty_lines = line_diff as isize - 1 - comments.len() as isize;
        if empty_lines > 0 {
            self.printer.hardbreak();
        }

        for comment in comments {
            self.printer.word("//");
            self.printer.word(comment.to_string());
            self.printer.hardbreak();
        }
    }
}
