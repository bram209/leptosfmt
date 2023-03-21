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

pub struct Formatter {
    pub printer: Printer,
    pub settings: FormatterSettings,
}

impl Formatter {
    pub fn new(settings: FormatterSettings) -> Self {
        Self {
            printer: Printer::new((&settings).into()),
            settings,
        }
    }
}
