use rstml::node::{FnBinding, KeyedAttribute, KeyedAttributeValue, NodeAttribute, NodeBlock};
use syn::{spanned::Spanned, Expr, RangeLimits, Stmt};

use crate::{formatter::Formatter, AttributeValueBraceStyle as Braces};

use super::ExpressionFormatter;

impl Formatter<'_> {
    pub fn attribute(&mut self, attribute: &NodeAttribute, next_attribute: Option<&NodeAttribute>) {
        self.flush_comments(attribute.span().start().line - 1, false);
        match attribute {
            NodeAttribute::Attribute(k) => self.keyed_attribute(k, next_attribute),
            NodeAttribute::Block(b) => self.node_block(b),
        }
    }

    fn keyed_attribute(
        &mut self,
        attribute: &KeyedAttribute,
        next_attribute: Option<&NodeAttribute>,
    ) {
        self.node_name(&attribute.key);

        match &attribute.possible_value {
            KeyedAttributeValue::None => {}
            KeyedAttributeValue::Binding(binding) => self.attribute_binding(binding),
            KeyedAttributeValue::Value(expr) => {
                let formatter = self
                    .settings
                    .attr_values
                    .get(&attribute.key.to_string())
                    .copied();

                self.printer.word("=");
                self.attribute_value(&expr.value, formatter, next_attribute);
            }
        }
    }

    fn attribute_binding(&mut self, binding: &FnBinding) {
        self.printer.word("(");
        let mut iterator = binding.inputs.iter().peekable();
        while let Some(input) = iterator.next() {
            self.format_syn_pat(input);
            if iterator.peek().is_some() {
                self.printer.word(",");
                self.printer.space();
            }
        }
        self.printer.word(")");
    }

    fn attribute_value(
        &mut self,
        value: &Expr,
        formatter: Option<ExpressionFormatter>,
        next_attribute: Option<&NodeAttribute>,
    ) {
        match (self.settings.attr_value_brace_style, value, next_attribute) {
            (Braces::WhenRequired, syn::Expr::Block(_), Some(next))
                if is_spread_attribute(next) =>
            {
                // If the next attribute is a spread attribute, make sure that the braces are not stripped from the expression
                // to avoid an ambiguity in the parser (i.e. `foo=bar {..}` could be interpreted as initialization of a struct called `bar`, instead of two separate attributes)
                self.node_value_expr(value, false, true, formatter)
            }
            (Braces::Always, syn::Expr::Block(_), _) => {
                self.node_value_expr(value, false, false, formatter)
            }
            (Braces::AlwaysUnlessLit, syn::Expr::Block(_) | syn::Expr::Lit(_), _) => {
                self.node_value_expr(value, false, true, formatter)
            }
            (Braces::Always | Braces::AlwaysUnlessLit, _, _) => {
                self.printer.word("{");
                self.node_value_expr(value, false, false, formatter);
                self.printer.word("}");
            }
            (Braces::WhenRequired, _, _) => self.node_value_expr(value, true, true, formatter),
            (Braces::Preserve, _, _) => self.node_value_expr(value, false, false, formatter),
        }
    }
}

fn is_spread_attribute(attr: &NodeAttribute) -> bool {
    if let NodeAttribute::Block(NodeBlock::ValidBlock(block)) = attr {
        if let [Stmt::Expr(
            Expr::Range(syn::ExprRange {
                start: None,
                limits: RangeLimits::HalfOpen(..),
                end,
                ..
            }),
            None,
        )] = block.stmts.as_slice()
        {
            return matches!(end.as_deref(), None | Some(Expr::Path(_)));
        }
    }

    false
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
                formatter.attribute(&attr, None);
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
                formatter.attribute(&attr, None);
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
        assert_snapshot!(formatted, @r#"key="K-123""#);
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
        assert_snapshot!(f, @r#"alt={"test img"}"#);
    }

    #[test]
    fn key_value_expr_attr_always_unless_lit_braces() {
        // single expr without braces
        let f = format_attr_with_brace_style! { AlwaysUnlessLit => on:click=move |_| set_value(0) };
        assert_snapshot!(f, @"on:click={move |_| set_value(0)}");

        // single expr without braces
        let f = format_attr_with_brace_style! { AlwaysUnlessLit => foo=bar };
        assert_snapshot!(f, @"foo={bar}");

        // single expr with braces
        let f =
            format_attr_with_brace_style! { AlwaysUnlessLit => on:click={move |_| set_value(0)} };
        assert_snapshot!(f, @"on:click={move |_| set_value(0)}");

        // literal numeric value
        let f = format_attr_with_brace_style! { AlwaysUnlessLit => width={100} };
        assert_snapshot!(f, @"width=100");

        // literal string value
        let f = format_attr_with_brace_style! { AlwaysUnlessLit => alt="test img" };
        assert_snapshot!(f, @r#"alt="test img""#);
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
        assert_snapshot!(f, @r#"alt="test img""#);
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
        assert_snapshot!(f, @r#"alt="test img""#);
    }

    #[test]
    fn let_bindings_single() {
        let f = format_attribute! {   let(name)  };
        assert_snapshot!(f, @r#"let(name)"#)
    }

    #[test]
    fn let_bindings_multiple() {
        let f = format_attribute! { let(name, foo, bar) };
        assert_snapshot!(f, @r#"let(name, foo, bar)"#)
    }

    #[test]
    fn let_bindings_destructuring() {
        let f = format_attribute! {   let(Item { name, value })  };
        assert_snapshot!(f, @r#"let(Item { name, value })"#)
    }

    #[test]
    fn prop_spreading_unnamed() {
        let f = format_attribute! {  {..}  };
        assert_snapshot!(f, @r#"{..}"#)
    }

    #[test]
    fn prop_spreading_named() {
        let f = format_attribute! {  { ..some_props }  };
        assert_snapshot!(f, @r#"{..some_props}"#)
    }
}
