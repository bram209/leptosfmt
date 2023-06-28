use leptosfmt_prettyplease::MacroFormatter;

use crate::{Formatter, FormatterSettings, ViewMacro};

pub struct ViewMacroFormatter {
    settings: FormatterSettings,
}

impl ViewMacroFormatter {
    pub fn new(settings: FormatterSettings) -> Self {
        Self { settings }
    }
}

impl MacroFormatter for ViewMacroFormatter {
    fn format(&self, printer: &mut leptosfmt_pretty_printer::Printer, mac: &syn::Macro) -> bool {
        if !mac.path.is_ident("view") {
            return false;
        }

        let Some(m) = ViewMacro::try_parse(None, mac) else { return false; };

        let mut formatter = Formatter::new(self.settings, printer);
        formatter.view_macro(&m);
        true
    }
}
