use syn_rsx::NodeValueExpr;

use crate::{formatter::Formatter, source_file::format_expr_source};

impl Formatter {
    pub fn node_value_expr(
        &mut self,
        value: &NodeValueExpr,
        unwrap_single_expr_blocks: bool,
        unwrap_single_lit_blocks: bool,
    ) {
        // if single line expression, format as '{expr}' instead of '{ expr }' (prettyplease inserts a space)
        if let syn::Expr::Block(expr_block) = value.as_ref() {
            if expr_block.attrs.is_empty() {
                if let [syn::Stmt::Expr(single_expr)] = &expr_block.block.stmts[..] {
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
        let formatted = leptosfmt_prettyplease::unparse_expr(expr);
        let formatted = format_expr_source(&formatted, self.settings).unwrap_or(formatted);

        let left_aligned = matches!(expr, syn::Expr::Lit(_));
        let mut iter = formatted.lines().peekable();
        while let Some(line) = iter.next() {
            if left_aligned {
                self.printer.word(line.trim_start().to_owned());
            } else {
                self.printer.word(line.to_owned());
            };

            if iter.peek().is_some() {
                self.printer.hardbreak();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::formatter::*;
    use crate::test_helpers::element;

    macro_rules! format_element {
        ($($tt:tt)*) => {{
            let comment = element! { $($tt)* };
            let mut formatter = Formatter::new(FormatterSettings { max_width: 40, ..Default::default() });
            formatter.element(&comment);
            formatter.printer.eof()
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
