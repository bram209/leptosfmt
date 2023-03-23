use std::collections::HashMap;

use leptosfmt_pretty_printer::{Printer, PrinterSettings};

mod attribute;
mod element;
mod fragment;
mod mac;
mod node;

pub use mac::format_macro;

#[derive(Clone, Copy)]
pub struct FormatterSettings {
    // Maximum width of each line
    pub max_width: usize,

    // Number of spaces per tab
    pub tab_spaces: usize,
}

impl Default for FormatterSettings {
    fn default() -> Self {
        Self {
            max_width: 100,
            tab_spaces: 4,
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

pub struct Formatter<'source> {
    pub printer: Printer,
    pub settings: FormatterSettings,
    last_line_check: Option<usize>,
    comments: HashMap<usize, String>,
    source: Option<&'source str>,
}

impl<'a> Formatter<'a> {
    pub fn new(settings: FormatterSettings) -> Self {
        Self {
            printer: Printer::new((&settings).into()),
            settings,
            comments: HashMap::new(),
            last_line_check: None,
            source: None,
        }
    }

    pub fn with_source(source: &'a str, settings: FormatterSettings) -> Formatter<'a> {
        Self {
            printer: Printer::new((&settings).into()),
            settings,
            comments: source
                .lines()
                .enumerate()
                .filter_map(|(i, l)| l.split("//").nth(1).map(|l| (i, l.to_owned())))
                .collect(),
            last_line_check: None,
            source: Some(source),
        }
    }

    pub fn write_comments(&mut self, line_index: usize) {
        let last = self.last_line_check.unwrap_or(line_index);

        self.last_line_check = Some(line_index);

        let comments = (last..=line_index)
            .filter_map(|l| self.comments.remove(&l))
            .peekable();

        for comment in comments {
            self.printer.word("//");
            self.printer.word(comment);
            self.printer.hardbreak();
        }
    }
}
