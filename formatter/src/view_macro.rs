use std::collections::HashMap;

use leptosfmt_prettyplease::MacroFormatter;

use crate::{Formatter, FormatterSettings, FormatterState, ViewMacro};

pub struct ViewMacroFormatter<'a> {
    settings: FormatterSettings,
    state: FormatterState<'a>,
}

impl<'a> ViewMacroFormatter<'a> {
    pub fn new(settings: FormatterSettings, formatter: FormatterState<'a>) -> Self {
        Self {
            settings,
            state: formatter,
        }
    }
}

impl<'c> MacroFormatter for ViewMacroFormatter<'c>
where
    Self: 'c,
{
    fn format<'a, 'b>(
        &'a self,
        printer: &'b mut leptosfmt_pretty_printer::Printer,
        mac: &syn::Macro,
    ) -> bool
    where
        Self: 'a,
        'b: 'a,
    {
        if !mac.path.is_ident("view") {
            return false;
        }

        let Some(m) = ViewMacro::try_parse(None, mac) else { return false; };
        let mut formatter = Formatter {
            printer,
            settings: self.settings,
            state: self.state.clone(),
        };

        formatter.view_macro(&m);
        true
    }
}
