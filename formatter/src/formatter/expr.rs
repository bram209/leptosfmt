use leptosfmt_prettyplease::MacroFormatter;
use syn::ExprLit;
use syn_rsx::NodeValueExpr;

use crate::{formatter::Formatter, FormatterSettings, ViewMacro};

struct ViewMacroFormatter {
    settings: FormatterSettings,
}

impl MacroFormatter for ViewMacroFormatter {
    fn accept(&self, mac: &syn::Macro) -> bool {
        mac.path.is_ident("view")
    }

    fn format(&self, printer: &mut leptosfmt_pretty_printer::Printer, mac: &syn::Macro) {
        let mut formatter = Formatter::new(self.settings, printer);
        let m = ViewMacro::try_parse(None, mac).unwrap();
        formatter.view_macro(&m);
    }
}

impl Formatter<'_> {
    pub fn node_value_expr(
        &mut self,
        value: &NodeValueExpr,
        unwrap_single_expr_blocks: bool,
        unwrap_single_lit_blocks: bool,
    ) {
        // if single line expression, format as '{expr}' instead of '{ expr }' (prettyplease inserts a space)
        if let syn::Expr::Block(expr_block) = value.as_ref() {
            if expr_block.attrs.is_empty() {
                if let [syn::Stmt::Expr(single_expr, _)] = &expr_block.block.stmts[..] {
                    // wrap with braces and do NOT insert spaces
                    if unwrap_single_expr_blocks
                        || (unwrap_single_lit_blocks && matches!(single_expr, syn::Expr::Lit(_)))
                    {
                        self.expr(single_expr);
                    } else {
                        self.printer.word("{");
                        self.expr(single_expr);
                        self.printer.word("}");
                    }
                    return;
                }
            }
        }

        self.expr(value.as_ref())
    }

    fn expr(&mut self, expr: &syn::Expr) {
        if let syn::Expr::Lit(ExprLit {
            lit: syn::Lit::Str(_),
            ..
        }) = expr
        {
            use syn::__private::ToTokens;
            let source = expr.to_token_stream().to_string();
            let mut iter = source.lines().peekable();
            while let Some(line) = iter.next() {
                self.printer.word(line.trim_start().to_owned());

                if iter.peek().is_some() {
                    self.printer.hardbreak();
                }
            }
            return;
        }

        leptosfmt_prettyplease::unparse_expr(
            expr,
            self.printer,
            Some(&ViewMacroFormatter {
                settings: self.settings,
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::formatter::*;
    use crate::test_helpers::{element, format_with};

    macro_rules! format_element {
        ($($tt:tt)*) => {{
            let comment = element! { $($tt)* };
            let settings = FormatterSettings { max_width: 40, ..Default::default() };

            format_with(settings, |formatter| {
                formatter.element(&comment);
            })
        }};
    }

    #[test]
    fn multiline_string_as_child() {
        let formatted = format_element! { <div>
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit,
                    sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
                            Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
                    Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. 
                Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
            </div>
        };

        insta::assert_snapshot!(formatted, @r###"
        <div>
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit,
            sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
            Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
            Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. 
            Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
        </div>
        "###);
    }
}
