use std::collections::HashMap;

use crop::Rope;
use leptosfmt_prettyplease::MacroFormatter;

use crate::{Formatter, FormatterSettings, ViewMacro};

pub struct ViewMacroFormatter<'a> {
    settings: &'a FormatterSettings,
    source: Option<&'a Rope>,
    line_offset: &'a mut Option<usize>,
    comments: HashMap<usize, Option<String>>,
}

impl ViewMacroFormatter<'_> {
    pub fn new<'a>(
        settings: &'a FormatterSettings,
        source: Option<&'a Rope>,
        line_offset: &'a mut Option<usize>,
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

pub fn get_macro_full_path(mac: &syn::Macro) -> String {
    mac.path
        .segments
        .iter()
        .map(|path| path.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

impl MacroFormatter for ViewMacroFormatter<'_> {
    fn format(
        &mut self,
        printer: &mut leptosfmt_pretty_printer::Printer,
        mac: &syn::Macro,
    ) -> bool {
        let mut formatted = false;

        for macro_name in &self.settings.macro_names {
            if &get_macro_full_path(mac) != macro_name {
                continue;
            }

            let Some(m) = ViewMacro::try_parse(Default::default(), mac) else {
                continue;
            };

            let mut formatter = Formatter {
                printer,
                settings: self.settings,
                source: self.source,
                line_offset: *self.line_offset,
                whitespace_and_comments: self.comments.clone(),
            };

            formatter.view_macro(&m);
            formatted = true;
            *self.line_offset = formatter.line_offset;
        }

        formatted
    }
}
