use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::str::FromStr;

use crop::Rope;
use crop::RopeSlice;
use leptosfmt_pretty_printer::{Printer, PrinterSettings};

mod attribute;
mod element;
mod expr;
mod fragment;
mod mac;
mod node;

pub use mac::format_macro;
pub use mac::ViewMacro;
use proc_macro2::LineColumn;
use proc_macro2::Span;
use quote::ToTokens;
use serde::Deserialize;
use serde::Serialize;
use syn::spanned::Spanned;

use crate::line_column_to_byte;

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
    pub(crate) last_span: Option<Span>,
    pub(crate) source: Option<Rope>,
}

impl<'a> Formatter<'a> {
    pub fn new(settings: FormatterSettings, printer: &'a mut Printer) -> Self {
        Self {
            printer,
            settings,
            last_span: None,
            source: None,
        }
    }

    fn visit_span<T>(&mut self, spanned: T)
    where
        T: Spanned,
        T: ToTokens,
    {
        let span = spanned.span();

        if let (Some(source), Some(last_span)) = (self.source.borrow_mut(), self.last_span) {
            if last_span.end().line != span.start().line {
                let text = get_text_beween_spans(source, last_span.end(), span.start().line - 1);
                let mut printed_empty_line = false;

                for line in text.lines().skip(1) {
                    let line = line.to_string();
                    // TODO if last line, make sure to skip the first span.start().column characters (NOT bytes!)
                    if let Some(comment) = line.split("//").nth(1).map(str::trim) {
                        self.printer.word("// ");
                        self.printer.word(comment.to_owned());
                        self.printer.hardbreak();
                        printed_empty_line = false;
                    } else if line.is_empty() && !printed_empty_line {
                        self.printer.hardbreak();
                        printed_empty_line = true;
                    }
                }
            }
        }

        self.last_span = Some(span);
    }

    pub fn with_source(
        settings: FormatterSettings,
        printer: &'a mut Printer,
        source: Rope,
    ) -> Self {
        Self {
            printer,
            settings,
            last_span: None,
            source: Some(source),
        }
    }

    fn tokens<T>(&mut self, tokens: &T)
    where
        T: ToTokens,
        T: Spanned,
    {
        self.visit_span(tokens);
        self.printer.word(tokens.to_token_stream().to_string())
    }
}

fn get_text_beween_spans(rope: &Rope, start: LineColumn, end_line: usize) -> RopeSlice<'_> {
    let start_byte = line_column_to_byte(rope, start);
    let end_byte = rope.byte_of_line(end_line) + rope.line(end_line).byte_len();
    return rope.byte_slice(start_byte..end_byte);
}
