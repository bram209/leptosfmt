use syn::{Block, Expr, ExprBlock, ExprLit, LitStr};

use crate::{formatter::Formatter, view_macro::ViewMacroFormatter};

impl Formatter<'_> {
    pub fn literal_str(&mut self, string: &LitStr) {
        let val = format!("\"{}\"", string.value());
        self.printer.word(val);
    }

    pub fn node_value_block(
        &mut self,
        block: &Block,
        unwrap_single_expr_blocks: bool,
        unwrap_single_lit_blocks: bool,
    ) {
        if let [syn::Stmt::Expr(single_expr, None)] = &block.stmts[..] {
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

        self.expr(&Expr::Block(ExprBlock {
            attrs: vec![],
            label: None,
            block: block.clone(),
        }))
    }
    pub fn node_value_expr(
        &mut self,
        value: &syn::Expr,
        unwrap_single_expr_blocks: bool,
        unwrap_single_lit_blocks: bool,
    ) {
        // if single line expression, format as '{expr}' instead of '{ expr }' (prettyplease inserts a space)
        if let syn::Expr::Block(expr_block) = value {
            if expr_block.attrs.is_empty() {
                return self.node_value_block(
                    &expr_block.block,
                    unwrap_single_expr_blocks,
                    unwrap_single_lit_blocks,
                );
            }
        }

        self.expr(value)
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
            Some(&ViewMacroFormatter::new(self.settings)),
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
