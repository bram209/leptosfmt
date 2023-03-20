use syn_rsx::{NodeAttribute};

use crate::formatter::Formatter;

impl Formatter {
    pub fn attribute(&mut self, attribute: &NodeAttribute) {
        self.node_name(&attribute.key);

        if let Some(value) = &attribute.value {
            self.printer.word("=");
            self.node_value(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        formatter::{Formatter, FormatterSettings},
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

    #[test]
    fn key_only_attr() {
        let formatted = format_attribute! { test };
        assert_eq!(formatted, "test");
    }

    #[test]
    fn key_only_dash_attr() {
        let formatted = format_attribute! { test-dash };
        assert_eq!(formatted, "test-dash");
    }

    #[test]
    fn key_value_integer_attr() {
        let formatted = format_attribute! { key=123 };
        assert_eq!(formatted, "key=123");
    }

    #[test]
    fn key_value_str_attr() {
        let formatted = format_attribute! { key="K-123" };
        assert_eq!(formatted, r#"key="K-123""#);
    }

    #[test]
    fn key_value_expr_attr() {
        let formatted = format_attribute! { on:click= move |_| set_value(0) };
        assert_eq!(formatted, r#"on:click=move |_| set_value(0)"#);
    }
}
