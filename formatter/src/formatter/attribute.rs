use rstml::node::{KeyedAttribute, NodeAttribute};
use syn::Expr;

use crate::{formatter::Formatter, AttributeValueBraceStyle as Braces};

impl Formatter<'_> {
    pub fn attribute(&mut self, attribute: &NodeAttribute) {
        match attribute {
            NodeAttribute::Attribute(k) => self.keyed_attribute(k),
            NodeAttribute::Block(b) => self.node_block(b),
        }
    }

    pub fn keyed_attribute(&mut self, attribute: &KeyedAttribute) {
        self.node_name(&attribute.key);

        if let Some(value) = attribute.value() {
            self.printer.word("=");
            self.attribute_value(value);
        }
    }

    fn attribute_value(&mut self, value: &Expr) {
        match (self.settings.attr_value_brace_style, value) {
            (Braces::Always, syn::Expr::Block(_)) => self.node_value_expr(value, false, false),
            (Braces::AlwaysUnlessLit, syn::Expr::Block(_) | syn::Expr::Lit(_)) => {
                self.node_value_expr(value, false, true)
            }
            (Braces::Always | Braces::AlwaysUnlessLit, _) => {
                self.printer.word("{");
                self.node_value_expr(value, false, false);
                self.printer.word("}");
            }
            (Braces::WhenRequired, _) => self.node_value_expr(value, true, true),
            (Braces::Preserve, _) => self.node_value_expr(value, false, false),
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::{
        formatter::{AttributeValueBraceStyle, FormatterSettings},
        test_helpers::{attribute, format_with},
    };

    macro_rules! format_attribute {
        ($($tt:tt)*) => {{
            let attr = attribute! { $($tt)* };
            format_with(FormatterSettings::default(), |formatter| {
                formatter.attribute(&attr);
            })
        }};
    }

    macro_rules! format_attr_with_brace_style {
        ($style:ident => $($tt:tt)*) => {{
            let attr = attribute! { $($tt)* };
            let settings = FormatterSettings {
                attr_value_brace_style:  AttributeValueBraceStyle:: $style,
                ..FormatterSettings::default()
            };

            format_with(settings, |formatter| {
                formatter.attribute(&attr);
            })
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
        assert_snapshot!(formatted, @"on:click=move |_| set_value(0)");
    }

    #[test]
    fn key_value_expr_attr_always_braces() {
        // sinle expr without braces
        let f = format_attr_with_brace_style! { Always => on:click=move |_| set_value(0) };
        assert_snapshot!(f, @"on:click={move |_| set_value(0)}");

        // single expr with braces
        let f = format_attr_with_brace_style! { Always => on:click={move |_| set_value(0)} };
        assert_snapshot!(f, @"on:click={move |_| set_value(0)}");

        // literal numeric value
        let f = format_attr_with_brace_style! { Always => width=100 };
        assert_snapshot!(f, @"width={100}");

        // literal string value
        let f = format_attr_with_brace_style! { Always => alt="test img" };
        assert_snapshot!(f, @r###"alt={"test img"}"###);
    }

    #[test]
    fn key_value_expr_attr_always_unless_lit_braces() {
        // sinle expr without braces
        let f = format_attr_with_brace_style! { AlwaysUnlessLit => on:click=move |_| set_value(0) };
        assert_snapshot!(f, @"on:click={move |_| set_value(0)}");

        // single expr with braces
        let f =
            format_attr_with_brace_style! { AlwaysUnlessLit => on:click={move |_| set_value(0)} };
        assert_snapshot!(f, @"on:click={move |_| set_value(0)}");

        // literal numeric value
        let f = format_attr_with_brace_style! { AlwaysUnlessLit => width={100} };
        assert_snapshot!(f, @"width=100");

        // literal string value
        let f = format_attr_with_brace_style! { AlwaysUnlessLit => alt="test img" };
        assert_snapshot!(f, @r###"alt="test img""###);
    }

    #[test]
    fn key_value_expr_attr_preserve_braces() {
        // single expr without braces
        let f = format_attr_with_brace_style! { Preserve => on:click=move |_| set_value(0) };
        assert_snapshot!(f, @"on:click=move |_| set_value(0)");

        // single expr with braces
        let f = format_attr_with_brace_style! { Preserve => on:click={move |_| set_value(0)} };
        assert_snapshot!(f, @"on:click={move |_| set_value(0)}");

        // literal numeric value with braces
        let f = format_attr_with_brace_style! { Preserve => width={100} };
        assert_snapshot!(f, @"width={100}");

        // literal string value without braces
        let f = format_attr_with_brace_style! { Preserve => alt="test img" };
        assert_snapshot!(f, @r###"alt="test img""###);
    }

    #[test]
    fn key_value_expr_attr_braces_when_required() {
        // single expr without braces
        let f = format_attr_with_brace_style! { WhenRequired => on:click=move |_| set_value(0) };
        assert_snapshot!(f, @"on:click=move |_| set_value(0)");

        // single expr with braces
        let f = format_attr_with_brace_style! { WhenRequired => on:click={move |_| set_value(0)} };
        assert_snapshot!(f, @"on:click=move |_| set_value(0)");

        // literal numeric value
        let f = format_attr_with_brace_style! { WhenRequired => width={100} };
        assert_snapshot!(f, @"width=100");

        // literal string value
        let f = format_attr_with_brace_style! { WhenRequired => alt={"test img"} };
        assert_snapshot!(f, @r###"alt="test img""###);
    }
}
