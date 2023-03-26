use syn_rsx::{NodeAttribute, NodeValueExpr};

use crate::{formatter::Formatter, AttributeValueBraceStyle};

impl Formatter {
    pub fn attribute(&mut self, attribute: &NodeAttribute) {
        self.node_name(&attribute.key);

        if let Some(value) = &attribute.value {
            self.printer.word("=");
            self.attribute_value(value);
        }
    }

    fn attribute_value(&mut self, value: &NodeValueExpr) {
        match self.settings.attr_value_brace_style {
            AttributeValueBraceStyle::Always => match value.as_ref() {
                syn::Expr::Block(_) => {
                    self.node_value_expr(value, false);
                }
                syn::Expr::Lit(_) => {
                    self.node_value_expr(value, true);
                }
                _ => {
                    self.printer.word("{");
                    self.node_value_expr(value, false);
                    self.printer.word("}");
                }
            },
            AttributeValueBraceStyle::WhenRequired => self.node_value_expr(value, true),
            AttributeValueBraceStyle::Preserve => self.node_value_expr(value, false),
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::{
        formatter::{AttributeValueBraceStyle, Formatter, FormatterSettings},
        test_helpers::attribute,
    };

    macro_rules! format_attribute {
        ($($tt:tt)*) => {{
            let attr = attribute! { $($tt)* };
            let mut formatter = Formatter::new(FormatterSettings::default());
            formatter.attribute(&attr);
            formatter.printer.eof()
        }};
    }

    macro_rules! format_attr_with_brace_style {
        ($style:ident => $($tt:tt)*) => {{
            let attr = attribute! { $($tt)* };
            let mut formatter = Formatter::new(FormatterSettings {
                attr_value_brace_style:  AttributeValueBraceStyle:: $style,
                ..FormatterSettings::default()
        });
            formatter.attribute(&attr);
            formatter.printer.eof()
        }};
    }

    #[test]
    fn key_only_attr() {
        let formatted = format_attribute! { test };
        assert_snapshot!(formatted, @"test");
    }

    #[test]
    fn key_only_dash_attr() {
        let formatted = format_attribute! { test-dash };
        assert_snapshot!(formatted, @"test-dash");
    }

    #[test]
    fn key_value_integer_attr() {
        let formatted = format_attribute! { key=123 };
        assert_snapshot!(formatted, @"key=123");
    }

    #[test]
    fn key_value_str_attr() {
        let formatted = format_attribute! { key="K-123" };
        assert_snapshot!(formatted, @r###"key="K-123""###);
    }

    #[test]
    fn key_value_expr_attr() {
        let formatted = format_attribute! { on:click= move |_| set_value(0) };
        assert_snapshot!(formatted, @"on:click={move |_| set_value(0)}");
    }

    #[test]
    fn key_value_expr_attr_always_braces() {
        // sinle expr without braces
        let f = format_attr_with_brace_style! { Always => on:click=move |_| set_value(0) };
        assert_snapshot!(f, @"on:click={move |_| set_value(0)}");

        // single expr with braces
        let f = format_attr_with_brace_style! { Always => on:click={move |_| set_value(0)} };
        assert_snapshot!(f, @"on:click={move |_| set_value(0)}");
    }

    #[test]
    fn key_value_expr_attr_preserve_braces() {
        // single expr without braces
        let f = format_attr_with_brace_style! { Preserve => on:click=move |_| set_value(0) };
        assert_snapshot!(f, @"on:click=move |_| set_value(0)");

        // single expr with braces
        let f = format_attr_with_brace_style! { Preserve => on:click={move |_| set_value(0)} };
        assert_snapshot!(f, @"on:click={move |_| set_value(0)}");
    }

    #[test]
    fn key_value_expr_attr_braces_when_required() {
        // single expr without braces
        let f = format_attr_with_brace_style! { WhenRequired => on:click=move |_| set_value(0) };
        assert_snapshot!(f, @"on:click=move |_| set_value(0)");

        // single expr with braces
        let f = format_attr_with_brace_style! { WhenRequired => on:click={move |_| set_value(0)} };
        assert_snapshot!(f, @"on:click=move |_| set_value(0)");
    }
}
