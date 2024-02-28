use crop::Rope;
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    File, Macro,
};

use crate::{ParentIndent, ViewMacro};

struct ViewMacroVisitor<'ast> {
    macros: Vec<ViewMacro<'ast>>,
    source: Rope,
    html_macro: String,
}

impl<'ast> Visit<'ast> for ViewMacroVisitor<'ast> {
    fn visit_macro(&mut self, node: &'ast Macro) {
        if node.path.is_ident(&self.html_macro) {
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
    html_macro: String,
) -> (Rope, Vec<ViewMacro<'_>>) {
    let mut visitor = ViewMacroVisitor {
        source,
        macros: Vec::new(),
        html_macro,
    };

    visitor.visit_file(file);
    (visitor.source, visitor.macros)
}
