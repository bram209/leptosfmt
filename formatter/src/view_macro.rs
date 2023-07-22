use crop::Rope;
use leptosfmt_prettyplease::MacroFormatter;
use proc_macro2::Span;

use crate::{Formatter, FormatterSettings, ViewMacro};

pub struct ViewMacroFormatter<'a> {
    settings: FormatterSettings,
    source: Option<&'a Rope>,
    last_span: Option<Span>,
}

impl ViewMacroFormatter<'_> {
    pub fn new<'a>(
        settings: FormatterSettings,
        source: Option<&'a Rope>,
        last_span: Option<Span>,
    ) -> ViewMacroFormatter<'a> {
        ViewMacroFormatter {
            settings,
            source,
            last_span,
        }
    }
}

impl MacroFormatter for ViewMacroFormatter<'_> {
    fn format(&self, printer: &mut leptosfmt_pretty_printer::Printer, mac: &syn::Macro) -> bool {
        if !mac.path.is_ident("view") {
            return false;
        }

        let Some(m) = ViewMacro::try_parse(None, mac) else { return false; };
        let mut formatter = Formatter {
            printer,
            settings: self.settings,
            source: self.source,
            last_span: self.last_span,
        };

        formatter.view_macro(&m);
        true
    }
}
