use std::{collections::HashMap, hash::Hash};

use leptosfmt_prettyplease::MacroFormatter;

use crate::{Formatter, FormatterSettings, ViewMacro};

pub struct ViewMacroFormatter<'a> {
    settings: FormatterSettings,
    comments: HashMap<usize, &'a str>,
}

impl<'a> ViewMacroFormatter<'a> {
    pub fn new(settings: FormatterSettings, comments: HashMap<usize, &'a str>) -> Self {
        Self { settings, comments }
    }
}

impl MacroFormatter for ViewMacroFormatter<'_> {
    fn format(&self, printer: &mut leptosfmt_pretty_printer::Printer, mac: &syn::Macro) -> bool {
        if !mac.path.is_ident("view") {
            return false;
        }

        let Some(m) = ViewMacro::try_parse(None, mac) else { return false; };

        let mut formatter = Formatter::new(self.settings, printer, self.comments.clone());
        formatter.view_macro(&m);
        true
    }
}
