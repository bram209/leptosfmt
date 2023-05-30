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
}

impl<'a> Formatter<'a> {
    pub fn new(settings: FormatterSettings, printer: &'a mut Printer) -> Self {
        Self { printer, settings }
    }
}
