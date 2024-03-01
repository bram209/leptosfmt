use std::collections::HashMap;

use crop::Rope;
use leptosfmt_prettyplease::MacroFormatter;

use crate::{Formatter, FormatterSettings, ViewMacro};

pub struct ViewMacroFormatter<'a> {
    settings: &'a FormatterSettings,
    source: Option<&'a Rope>,
    line_offset: Option<usize>,
    comments: HashMap<usize, Option<String>>,
}

impl ViewMacroFormatter<'_> {
    pub fn new<'a>(
        settings: &'a FormatterSettings,
        source: Option<&'a Rope>,
        line_offset: Option<usize>,
        comments: HashMap<usize, Option<String>>,
    ) -> ViewMacroFormatter<'a> {
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
        for format_macro in &self.settings.format_macros {
            if !mac.path.is_ident(&format_macro) {
                continue;
            }

            let Some(m) = ViewMacro::try_parse(Default::default(), mac, format_macro.to_owned())
            else {
                continue;
            };
            let mut formatter = Formatter {
                printer,
                settings: self.settings,
                source: self.source,
                line_offset: self.line_offset,
                whitespace_and_comments: self.comments.clone(),
            };

            formatter.view_macro(&m);
        }

        true
    }
}
