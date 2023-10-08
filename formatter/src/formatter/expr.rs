use std::collections::HashMap;

use syn::{spanned::Spanned, Block, Expr, ExprBlock, ExprLit, LitStr};

use crate::{formatter::Formatter, view_macro::ViewMacroFormatter};

fn trim_start_with_max(str: &str, max_chars: usize) -> &str {
    let mut chars = 0;
    str.trim_start_matches(|c: char| {
        if c.is_whitespace() {
            chars += 1;
            chars <= max_chars
        } else {
            false
        }
    })
}

impl Formatter<'_> {
    pub fn string(&mut self, string: &str, start_column: usize) {
        let mut iter = string.lines().enumerate().peekable();
        while let Some((line_num, line)) = iter.next() {
            if line_num == 0 {
                self.printer.word(line.to_string())
            } else {
                self.printer
                    .word(trim_start_with_max(line, start_column).to_string());
            }

            if iter.peek().is_some() {
                self.printer.hardbreak();
            }
        }
    }

    pub fn literal_str(&mut self, lit_str: &LitStr) {
        self.printer.word("\"");
        let string = lit_str.value();

        let start_span = lit_str.span().start();
        self.string(&string, start_span.column);
        self.printer.word("\"");
    }

    pub fn node_value_block_expr(
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
                return self.node_value_block_expr(
                    &expr_block.block,
                    unwrap_single_expr_blocks,
                    unwrap_single_lit_blocks,
                );
            }
        }

        self.expr(value)
    }

    fn expr(&mut self, expr: &syn::Expr) {
        let span = expr.span();
        self.flush_comments(span.start().line - 1);
        if let syn::Expr::Lit(ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) = expr
        {
            self.literal_str(lit_str);
            return;
        }

        let start_line = span.start().line - 1;
        let end_line = span.end().line - 1;

        let cmt_or_wp_lines: Vec<usize> = self
            .whitespace_and_comments
            .iter()
            .filter(|(line, _comment)| **line >= start_line && **line < end_line)
            .map(|(line, _)| *line)
            .collect();

        let comments_or_whitespace = cmt_or_wp_lines
            .into_iter()
            .map(|line| (line, self.whitespace_and_comments.remove(&line).unwrap()))
            .collect::<HashMap<_, _>>();

        leptosfmt_prettyplease::unparse_expr(
            expr,
            self.printer,
            Some(&ViewMacroFormatter::new(
                self.settings,
                self.source,
                self.line_offset,
                comments_or_whitespace,
            )),
        );
    }
}

#[cfg(test)]
mod tests {
    use rstml::node::Node;

    use crate::formatter::*;
    use crate::test_helpers::{element_from_string, format_with};

    macro_rules! format_element {
        ($($tt:tt)*) => {{
            let comment = element_from_string! { $($tt)* };
            let settings = FormatterSettings { max_width: 40, ..Default::default() };

            format_with(settings, |formatter| {
                formatter.node(&Node::Element(comment));
            })
        }};
    }

    #[test]
    fn multiline_string_as_child() {
        let formatted = format_element! {r#"<div>
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit,
                        sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
                                Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
                        Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
                    Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
            </div>"#};

        insta::assert_snapshot!(formatted, @r#"
        <div>
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit,
                sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
                        Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
                Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
            Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
        </div>
        "#);
    }

    #[test]
    fn string_whitespace_prefix() {
        let formatted = format_element! {r#"<div>
                    "    foo"
            </div>"#};

        insta::assert_snapshot!(formatted, @r#"
        <div>"    foo"</div>
        "#);
    }

    #[test]
    fn multiline_string_whitespace_prefix() {
        let formatted = format_element! {r#"<div>
                    "        Lorem ipsum dolor sit amet, consectetur adipiscing elit,
                        sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
                                Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
                        Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
                    Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
            </div>"#};

        insta::assert_snapshot!(formatted, @r#"
        <div>
            "        Lorem ipsum dolor sit amet, consectetur adipiscing elit,
                sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
                        Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
                Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
            Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
        </div>
        "#);
    }

    #[test]
    fn multiline_raw_string_as_child() {
        let formatted = format_element! {r#"<div>
                    Lorem ipsum dolor sit amet, consectetur adipiscing elit,
                        sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
                                Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
                        Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
                    Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
            </div>"#};

        insta::assert_snapshot!(formatted, @r###"
        <div>
            Lorem ipsum dolor sit amet, consectetur adipiscing elit,
                sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
                        Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
                Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
            Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
        </div>
        "###);
    }

    #[test]
    fn test_raws() {
        let formatted = format_element!(
            r#"<div>
        <div class=format!("grid grid-cols-4 gap-1 {extend_tw_classes}")>
        <button
            class="hover:bg-blue-300 bg-slate-400 mt-6 rounded-md border-cyan-500 border-2 drop-shadow-lg"
            on:click=first
        >
              First
        </button>

        <button
            class="hover:bg-blue-300 bg-slate-400 mt-6 rounded-md border-cyan-500 border-2 drop-shadow-lg"
            on:click=previous
        >
              Previous
        </button>

        <button
            class="hover:bg-blue-300 bg-slate-400 mt-6 rounded-md border-cyan-500 border-2 drop-shadow-lg"
            on:click=next
        >
              Next
        </button>
        </div></div>"#
        );

        insta::assert_snapshot!(formatted, @r#"
        <div>
            <div class=format!(
                "grid grid-cols-4 gap-1 {extend_tw_classes}",
            )>
                <button
                    class="hover:bg-blue-300 bg-slate-400 mt-6 rounded-md border-cyan-500 border-2 drop-shadow-lg"
                    on:click=first
                >
                    First
                </button>
                <button
                    class="hover:bg-blue-300 bg-slate-400 mt-6 rounded-md border-cyan-500 border-2 drop-shadow-lg"
                    on:click=previous
                >
                    Previous
                </button>
                <button
                    class="hover:bg-blue-300 bg-slate-400 mt-6 rounded-md border-cyan-500 border-2 drop-shadow-lg"
                    on:click=next
                >
                    Next
                </button>
            </div>
        </div>
        "#);
    }
}
