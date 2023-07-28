use proc_macro2::LineColumn;
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    Expr, File, Macro,
};

use crate::ViewMacro;

#[derive(Default)]
struct ViewMacroVisitor<'ast> {
    indent_stack: Vec<LineColumn>,
    macros: Vec<ViewMacro<'ast>>,
}

impl<'ast> Visit<'ast> for ViewMacroVisitor<'ast> {
    fn visit_stmt(&mut self, i: &'ast syn::Stmt) {
        self.indent_stack.push(i.span().start());
        visit::visit_stmt(self, i);
        self.indent_stack.pop();
    }

    fn visit_expr(&mut self, i: &'ast Expr) {
        self.indent_stack.push(i.span().start());
        visit::visit_expr(self, i);
        self.indent_stack.pop();
    }

    fn visit_arm(&mut self, i: &'ast syn::Arm) {
        self.indent_stack.push(i.span().start());
        visit::visit_arm(self, i);
        self.indent_stack.pop();
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        if node.path.is_ident("view") {
            let span_start = node.span().start().column;
            let span_line = node.span().start().line;
            let indent = self
                .indent_stack
                .iter()
                .filter(|v| v.line == span_line && v.column < span_start)
                .map(|v| v.column)
                .min()
                .unwrap_or(
                    self.indent_stack
                        .iter()
                        .rev()
                        .find(|v| v.column < span_start)
                        .map(|i| i.column)
                        .unwrap_or(0),
                );

            let rustfmt_indent = detect_rustfmt_indent(&self.indent_stack);
            if let Some(view_mac) =
                ViewMacro::try_parse(Some(indent + rustfmt_indent.unwrap_or(0)), node)
            {
                self.macros.push(view_mac);
            }
        }

        visit::visit_macro(self, node);
    }
}

fn detect_rustfmt_indent(stack: &[LineColumn]) -> Option<usize> {
    if let [lc] = stack {
        return Some(lc.column);
    }

    let mut iter = stack.iter().peekable();
    while let Some(lc) = iter.next() {
        if let Some(next) = iter.peek() {
            if next.line > lc.line && next.column > lc.column {
                return Some(next.column - lc.column);
            }
        }
    }

    None
}

pub fn collect_macros_in_file(file: &File) -> Vec<ViewMacro<'_>> {
    let mut visitor = ViewMacroVisitor::default();
    visitor.visit_file(file);
    visitor.macros
}
