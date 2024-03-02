use crop::Rope;
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    File, Macro,
};

use crate::{view_macro::get_macro_full_path, ParentIndent, ViewMacro};

struct ViewMacroVisitor<'ast> {
    macros: Vec<ViewMacro<'ast>>,
    source: Rope,
    macro_names: Vec<String>,
}

impl<'ast> Visit<'ast> for ViewMacroVisitor<'ast> {
    fn visit_macro(&mut self, node: &'ast Macro) {
        let should_format = self
            .macro_names
            .iter()
            .any(|macro_name| &get_macro_full_path(node) == macro_name);

        if should_format {
            let span_line = node.span().start().line;
            let line = self.source.line(span_line - 1);

            let indent_chars: Vec<_> = line
                .chars()
                .take_while(|&c| c == ' ' || c == '\t')
                .collect();

            let tabs = indent_chars.iter().filter(|&&c| c == '\t').count();
            let spaces = indent_chars.iter().filter(|&&c| c == ' ').count();

            if let Some(view_mac) = ViewMacro::try_parse(ParentIndent { tabs, spaces }, node) {
                self.macros.push(view_mac);
            }
        }

        visit::visit_macro(self, node);
    }
}

pub fn collect_macros_in_file(
    file: &File,
    source: Rope,
    macro_names: Vec<String>,
) -> (Rope, Vec<ViewMacro<'_>>) {
    let mut visitor = ViewMacroVisitor {
        source,
        macros: Vec::new(),
        macro_names,
    };

    visitor.visit_file(file);
    (visitor.source, visitor.macros)
}
