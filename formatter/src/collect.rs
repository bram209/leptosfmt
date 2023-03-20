use syn::{
    visit::{self, Visit},
    Expr, File, Macro,
};

#[derive(Default)]
struct ViewMacroVisitor<'ast> {
    macros: Vec<&'ast Macro>,
}

impl<'ast> Visit<'ast> for ViewMacroVisitor<'ast> {
    fn visit_macro(&mut self, node: &'ast Macro) {
        if node.path.is_ident("view") {
            self.macros.push(node);
        }

        visit::visit_macro(self, node);
    }
}

pub fn collect_macros_in_file(file: &File) -> Vec<&Macro> {
    let mut visitor = ViewMacroVisitor::default();
    visitor.visit_file(file);
    visitor.macros
}

pub fn collect_macros_in_expr(expr: &Expr) -> Vec<&Macro> {
    let mut visitor = ViewMacroVisitor::default();
    visitor.visit_expr(expr);
    visitor.macros
}
