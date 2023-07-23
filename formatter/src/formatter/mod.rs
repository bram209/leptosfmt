use std::borrow::BorrowMut;

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
    pub(crate) source: Option<&'a Rope>,
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

    fn visit_span(&mut self, span: Span) {
        if let (Some(source), Some(last_span)) = (self.source.borrow_mut(), self.last_span) {
            if last_span.end().line != span.start().line {
                let text = get_text_beween_spans(source, last_span.end(), span.start());
                let mut printed_empty_line = false;

                let mut lines = text.lines().map(|line| line.to_string());
                let first_line = lines.next().unwrap();
                if first_line.trim().starts_with("//") {
                    panic!(
                        "End of line comments are not supported yet (at line {}): {:?}",
                        last_span.end().line,
                        first_line
                    );
                }

                for line in lines {
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

    fn visit_spanned<T>(&mut self, spanned: T)
    where
        T: Spanned,
    {
        let span = spanned.span();
        self.visit_span(span);
    }

    pub fn with_source(
        settings: FormatterSettings,
        printer: &'a mut Printer,
        source: &'a Rope,
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
        self.visit_spanned(tokens);
        self.printer.word(tokens.to_token_stream().to_string())
    }
}

fn get_text_beween_spans(rope: &Rope, start: LineColumn, end: LineColumn) -> RopeSlice<'_> {
    let start_byte = line_column_to_byte(rope, start);
    let end_byte = line_column_to_byte(rope, end);

    return rope.byte_slice(start_byte..end_byte);
}
