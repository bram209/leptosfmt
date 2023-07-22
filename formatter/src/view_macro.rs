use crop::Rope;
use leptosfmt_prettyplease::MacroFormatter;
use proc_macro2::Span;

use crate::{Formatter, FormatterSettings, ViewMacro};

pub struct ViewMacroFormatter {
    settings: FormatterSettings,
    source: Option<Rope>,
    last_span: Option<Span>,
}

impl ViewMacroFormatter {
    pub fn new(settings: FormatterSettings, source: Option<Rope>, last_span: Option<Span>) -> Self {
        Self {
            settings,
            source,
            last_span,
        }
    }
}

impl MacroFormatter for ViewMacroFormatter {
    fn format(&self, printer: &mut leptosfmt_pretty_printer::Printer, mac: &syn::Macro) -> bool {
        if !mac.path.is_ident("view") {
            return false;
        }

        let Some(m) = ViewMacro::try_parse(None, mac) else { return false; };
        let mut formatter = Formatter {
            printer,
            settings: self.settings,
            source: self.source.clone(),
            last_span: self.last_span,
        };

        formatter.view_macro(&m);
        true
    }
}
