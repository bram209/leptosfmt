use crop::Rope;
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    File, Macro,
};

use crate::ViewMacro;

struct ViewMacroVisitor<'ast> {
    macros: Vec<ViewMacro<'ast>>,
    source: Rope,
}

impl<'ast> Visit<'ast> for ViewMacroVisitor<'ast> {
    fn visit_macro(&mut self, node: &'ast Macro) {
        if node.path.is_ident("view") {
            let span_line = node.span().start().line;
            let indent = self
                .source
                .line(span_line - 1)
                .chars()
                .take_while(|&c| c == ' ')
                .count();

            if let Some(view_mac) = ViewMacro::try_parse(Some(indent), node) {
                self.macros.push(view_mac);
            }
        }

        visit::visit_macro(self, node);
    }
}

pub fn collect_macros_in_file(file: &File, source: Rope) -> (Rope, Vec<ViewMacro<'_>>) {
    let mut visitor = ViewMacroVisitor {
        source,
        macros: Vec::new(),
    };

    visitor.visit_file(file);
    (visitor.source, visitor.macros)
}
