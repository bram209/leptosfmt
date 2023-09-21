use std::collections::HashMap;

use crop::Rope;

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
            crlf_line_endings: false,
        }
    }
}

pub struct Formatter<'a> {
    pub printer: &'a mut leptosfmt_pretty_printer::Printer,
    pub settings: FormatterSettings,
    pub(crate) source: Option<&'a Rope>,
    pub(crate) whitespace_and_comments: HashMap<usize, Option<String>>,
    pub(crate) line_offset: Option<usize>,
}

impl<'a> Formatter<'a> {
    pub fn new(settings: FormatterSettings, printer: &'a mut Printer) -> Self {
        Self {
            printer,
            settings,
            source: None,
            whitespace_and_comments: HashMap::new(),
            line_offset: None,
        }
    }
    pub fn with_source(
        settings: FormatterSettings,
        printer: &'a mut Printer,
        source: &'a Rope,
        comments: HashMap<usize, Option<String>>,
    ) -> Self {
        Self {
            printer,
            settings,
            source: Some(source),
            whitespace_and_comments: comments,
            line_offset: None,
        }
    }

    pub fn trim_whitespace(&mut self, line_index: usize) {
        // keep removing whitespace until we reach the current line or a comment
        let last = self.line_offset.unwrap_or(0);

        for line in last..=line_index {
            if let Some(entry) = self.whitespace_and_comments.get(&line) {
                if entry.is_none() {
                    self.whitespace_and_comments.remove(&line);
                } else {
                    return;
                }
            }
        }
    }

    pub fn flush_comments(&mut self, line_index: usize) {
        let last = self.line_offset.unwrap_or(0);

        let comments_or_empty_lines: Vec<_> = (last..=line_index)
            .filter_map(|l| self.whitespace_and_comments.remove(&l))
            .collect();

        let mut prev_is_empty_line = false;

        for comment_or_empty in comments_or_empty_lines {
            if let Some(comment) = comment_or_empty {
                self.printer.word("// ");
                self.printer.word(comment);
                self.printer.hardbreak();
                prev_is_empty_line = false;
            } else if self.line_offset.is_some() {
                // Do not print multiple consecutive empty lines
                if !prev_is_empty_line {
                    self.printer.hardbreak();
                }

                prev_is_empty_line = true;
            }
        }

        self.line_offset = Some(line_index);
    }
}
