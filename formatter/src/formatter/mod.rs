use std::collections::HashMap;

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
    line_offset: Option<usize>,
    start_line_offset: Option<usize>,
    comments: HashMap<usize, Option<&'a str>>,
    last_span: Option<Span>,
    source: Option<&'a Rope>,
}

impl<'a> Formatter<'a> {
    pub fn new(
        settings: FormatterSettings,
        printer: &'a mut Printer,
        comments: HashMap<usize, Option<&'a str>>,
    ) -> Self {
        Self {
            printer,
            settings,
            comments,
            line_offset: None,
            start_line_offset: None,
            last_span: None,
            source: None,
        }
    }

    // fn token(&mut self, token: impl Token) {
    //     self.visit_span(token);
    // }

    fn visit_span<T>(&mut self, spanned: T)
    where
        T: Spanned,
        T: ToTokens,
    {
        dbg!(spanned.to_token_stream().to_string());
        let span = spanned.span();

        if let (Some(source), Some(last_span)) = (self.source, self.last_span) {
            if last_span.end().line != span.start().line {
                let text = get_text_beween_spans(source, last_span.end(), span.start().line - 1);
                dbg!(last_span.end(), span.start(), text);
                for (_idx, line) in text.lines().skip(1).enumerate() {
                    let line = line.to_string();
                    // TODO if last line, make sure to skip the first span.start().column characters (NOT bytes!)
                    let Some(comment) = line.split("//").nth(1).map(str::trim) else {
                        continue;
                    };

                    self.printer.word("// ");
                    self.printer.word(comment.to_owned());
                    self.printer.hardbreak();
                }
            }
        }

        self.last_span = Some(span);
    }

    pub fn with_source(
        settings: FormatterSettings,
        printer: &'a mut Printer,
        source: &'a Rope,
    ) -> Self {
        Self {
            printer,
            settings,
            comments: HashMap::new(), // source
            // .lines()
            // .enumerate()
            // .filter_map(|(i, l)| {
            //     if l.trim().is_empty() {
            //         Some((i, None))
            //     } else {
            //         l.split("//").nth(1).map(|l| (i, Some(l)))
            //     }
            // })
            // .collect(),
            line_offset: None,
            start_line_offset: None,
            last_span: None,
            source: Some(source),
        }
    }

    pub fn write_comments(&mut self, line_index: usize) {
        let last = self
            .line_offset
            .unwrap_or(self.start_line_offset.unwrap_or(0));

        let comments_or_empty_lines: Vec<_> = (last..=line_index)
            .filter_map(|l| self.comments.remove(&l))
            .collect();

        let mut prev_is_empty_line = false;

        for comment_or_empty in comments_or_empty_lines {
            if let Some(comment) = comment_or_empty {
                self.printer.word("//");
                self.printer.word(comment.to_string());
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

    dbg!(start, end_line);

    return rope.byte_slice(start_byte..end_byte);
}
