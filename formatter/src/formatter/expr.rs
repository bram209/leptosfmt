use std::collections::HashMap;

use syn::{spanned::Spanned, Block, Expr, ExprBlock, ExprLit, LitStr};

use crate::{formatter::Formatter, get_text_beween_spans, view_macro::ViewMacroFormatter};

use super::ExpressionFormatter;

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

    pub fn source_code<T: Spanned>(&mut self, span: &T) {
        let span = span.span();
        let source = self.source.unwrap();
        let code_fragment = get_text_beween_spans(source, span.start(), span.end()).to_string();
        self.string(&code_fragment, span.start().column)
    }

    pub fn literal_str(&mut self, lit_str: &LitStr) {
        if self.source.is_some() {
            self.source_code(lit_str);
            return;
        }

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
                self.expr(single_expr, None);
            } else {
                self.printer.word("{");
                self.expr(single_expr, None);
                self.printer.word("}");
            }
            return;
        }

        self.expr(
            &Expr::Block(ExprBlock {
                attrs: vec![],
                label: None,
                block: block.clone(),
            }),
            None,
        )
    }

    pub fn node_value_expr(
        &mut self,
        value: &syn::Expr,
        unwrap_single_expr_blocks: bool,
        unwrap_single_lit_blocks: bool,
        formatter: Option<ExpressionFormatter>,
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

        self.expr(value, formatter)
    }

    fn expr(&mut self, expr: &syn::Expr, formatter: Option<ExpressionFormatter>) {
        let span = expr.span();
        self.flush_comments(span.start().line - 1, false);
        if let syn::Expr::Lit(ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) = expr
        {
            if let Some(formatter) = formatter {
                formatter.format(self, lit_str.value())
            } else {
                self.literal_str(lit_str);
            }
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

    use crate::formatter::*;
    use crate::test_helpers::format_element_from_string;

    macro_rules! format_element {
        ($($tt:tt)*) => {{
            let settings = FormatterSettings {
                max_width: 40,
                ..Default::default()
            };

            let element = element! { $($tt)* };
            format_with(settings,|formatter| {
                formatter.node(&Node::Element(element));
            })
        }};
    }

    macro_rules! format_element_from_string {
        ($($tt:tt)*) => {{
            let settings = FormatterSettings {
                max_width: 40,
                ..Default::default()
            };

            format_element_from_string(settings, $($tt)*)
        }};
    }

    #[test]
    fn multiline_string_as_child() {
        let formatted = format_element_from_string! {r#"<div>
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
        let formatted = format_element_from_string! {r#"<div>
                    "    foo"
            </div>"#};

        insta::assert_snapshot!(formatted, @r#"
        <div>"    foo"</div>
        "#);
    }

    #[test]
    fn multiline_string_whitespace_prefix() {
        let formatted = format_element_from_string! {r#"<div>
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
    fn multiline_unquoted_string_as_child() {
        let formatted = format_element_from_string! {r#"<div>
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
    fn raw_string_as_child() {
        let formatted = format_element_from_string!(r##"<p>r#"some" string"#</p>"##);

        insta::assert_snapshot!(formatted, @r##"
        <p>r#"some" string"#</p>
        "##);
    }

    #[test]
    fn unicode_scalar() {
        let formatted = format_element_from_string!(r#"<p>"\u{00A9}🦀"</p>"#);
        insta::assert_snapshot!(formatted, @r#"
        <p>"\u{00A9}🦀"</p>
        "#);
    }

    #[test]
    fn codeblock_with_empty_lines() {
        let formatted = format_element_from_string! { r#"
                    <h2>
                        {

                        }
                    </h2>
            "#
        };

        insta::assert_snapshot!(formatted, @r###"
        <h2>{}</h2>
        "###);
    }

    #[test]
    fn codeblock_body() {
        let formatted = format_element_from_string! { r#"<h2>
                        {match error_code {
                            StatusCode::SERVICE_UNAVAILABLE => "custom error msg".to_string(),
                            error_code => error_code.to_string(),
                        }}
                    </h2>"#
        };

        insta::assert_snapshot!(formatted, @r###"
        <h2>
            {match error_code {
                StatusCode::SERVICE_UNAVAILABLE => {
                    "custom error msg".to_string()
                }
                error_code => error_code.to_string(),
            }}
        </h2>
        "###);
    }
}
