use std::collections::HashMap;

use crop::Rope;
use leptosfmt_prettyplease::MacroFormatter;

use crate::{Formatter, FormatterSettings, ViewMacro};

pub struct ViewMacroFormatter<'a> {
    settings: FormatterSettings,
    source: Option<&'a Rope>,
    line_offset: Option<usize>,
    comments: HashMap<usize, Option<String>>,
}

impl ViewMacroFormatter<'_> {
    pub fn new(
        settings: FormatterSettings,
        source: Option<&Rope>,
        line_offset: Option<usize>,
        comments: HashMap<usize, Option<String>>,
    ) -> ViewMacroFormatter<'_> {
        ViewMacroFormatter {
            settings,
            source,
            line_offset,
            comments,
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
            line_offset: self.line_offset,
            whitespace_and_comments: self.comments.clone(),
        };

        formatter.view_macro(&m);
        true
    }
}
