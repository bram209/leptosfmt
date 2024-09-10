use std::collections::HashMap;
use std::fmt::Debug;

use crop::Rope;

use leptosfmt_pretty_printer::{Printer, PrinterSettings};

mod attribute;
mod element;
mod expr;
mod fragment;
mod mac;
mod node;
mod tailwind;

pub use mac::format_macro;
pub use mac::{ParentIndent, ViewMacro};

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum ClosingTagStyle {
    /// Preserve the original closing tag style (self-closing or a separate closing tag)
    Preserve,
    /// Self closing tag for elements with no children: `<div></div>` formats to `<div />`
    SelfClosing,
    /// Separate closing tag for elements with no children: `<div />` formats to `<div></div>`
    NonSelfClosing,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum AttributeValueBraceStyle {
    Always,
    AlwaysUnlessLit,
    WhenRequired,
    Preserve,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum IndentationStyle {
    Auto,
    Spaces,
    Tabs,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum NewlineStyle {
    Auto,
    Native,
    Unix,
    Windows,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum ExpressionFormatter {
    Tailwind,
}

impl ExpressionFormatter {
    pub fn format(&self, formatter: &mut Formatter, value: String) {
        match self {
            Self::Tailwind => formatter.tailwind_expr(value),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct FormatterSettings {
    /// Maximum width of each line
    pub max_width: usize,

    /// Number of spaces per tab
    pub tab_spaces: usize,

    /// Determines indentation style (tabs or spaces)
    pub indentation_style: IndentationStyle,

    /// Determines line ending (unix or windows)
    pub newline_style: NewlineStyle,

    /// Determines placement of braces around single expression attribute values
    pub attr_value_brace_style: AttributeValueBraceStyle,

    /// Preferred style for closing tags (self-closing or not) when a non-void element has no children
    pub closing_tag_style: ClosingTagStyle,

    /// Determines macros to be formatted. Default: leptos::view, view
    pub macro_names: Vec<String>,

    /// Determines whether to format attribute values with a specific formatter (e.g. tailwind)
    pub attr_values: HashMap<String, ExpressionFormatter>,
}

impl Default for FormatterSettings {
    fn default() -> Self {
        Self {
            max_width: 100,
            tab_spaces: 4,
            attr_value_brace_style: AttributeValueBraceStyle::WhenRequired,
            indentation_style: IndentationStyle::Auto,
            newline_style: NewlineStyle::Auto,
            closing_tag_style: ClosingTagStyle::Preserve,
            macro_names: vec!["leptos::view".to_string(), "view".to_string()],
            attr_values: HashMap::new(),
        }
    }
}

fn uses_crlf_line_ending(source: &Rope) -> bool {
    source
        .raw_lines()
        .next()
        .map(|raw_line| raw_line.to_string().ends_with("\r\n"))
        .unwrap_or_default()
}

fn uses_tabs_for_indentation(source: &Rope) -> bool {
    source
        .lines()
        .find(|line| matches!(line.chars().next(), Some('\t') | Some(' ')))
        .map(|line| matches!(line.chars().next(), Some('\t')))
        .unwrap_or_default()
}

impl FormatterSettings {
    pub fn to_printer_settings(&self, source: Option<&Rope>) -> PrinterSettings {
        PrinterSettings {
            margin: self.max_width as isize,
            tab_spaces: self.tab_spaces as isize,
            min_space: 60,
            crlf_line_endings: match self.newline_style {
                NewlineStyle::Auto => source.map(uses_crlf_line_ending).unwrap_or_default(),
                NewlineStyle::Native => cfg!(windows),
                NewlineStyle::Unix => false,
                NewlineStyle::Windows => true,
            },
            hard_tabs: match self.indentation_style {
                IndentationStyle::Auto => source.map(uses_tabs_for_indentation).unwrap_or_default(),
                IndentationStyle::Spaces => false,
                IndentationStyle::Tabs => true,
            },
        }
    }
}

pub struct Formatter<'a> {
    pub printer: &'a mut leptosfmt_pretty_printer::Printer,
    pub settings: &'a FormatterSettings,
    pub(crate) source: Option<&'a Rope>,
    pub(crate) whitespace_and_comments: HashMap<usize, Option<String>>,
    pub(crate) line_offset: Option<usize>,
}

impl<'a> Formatter<'a> {
    pub fn new(settings: &'a FormatterSettings, printer: &'a mut Printer) -> Self {
        Self {
            printer,
            settings,
            source: None,
            whitespace_and_comments: HashMap::new(),
            line_offset: None,
        }
    }
    pub fn with_source(
        settings: &'a FormatterSettings,
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

    pub fn flush_comments(&mut self, line_index: usize, skip_trailing_whitespace: bool) {
        let last = self.line_offset.unwrap_or(0);

        if last > line_index {
            return;
        }

        let comments_or_empty_lines: Vec<_> = (last..=line_index)
            .filter_map(|l| self.whitespace_and_comments.remove(&l))
            .collect();

        // If we need to skip trailing whitespace, calculate how many elements we need to take,
        // until no comments are left in the vector
        let take_n = if skip_trailing_whitespace {
            comments_or_empty_lines
                .iter()
                .rev()
                .position(Option::is_some)
                .map(|i| comments_or_empty_lines.len() - i)
        } else {
            None
        }
        .unwrap_or(comments_or_empty_lines.len());

        let mut prev_is_empty_line = false;

        for comment_or_empty in comments_or_empty_lines.into_iter().take(take_n) {
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
