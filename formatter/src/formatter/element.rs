use crate::formatter::Formatter;
use rstml::node::{Node, NodeAttribute, NodeElement};

impl Formatter<'_> {
    pub fn element(&mut self, element: &NodeElement) {
        let name = element.name().to_string();
        let is_void = is_void_element(&name, !element.children.is_empty());
        self.opening_tag(element, is_void);

        if !is_void {
            self.children(&element.children, element.attributes().len());
            self.closing_tag(element)
        }
    }

    fn opening_tag(&mut self, element: &NodeElement, is_void: bool) {
        self.tokens(&element.open_tag.token_lt);
        self.visit_span(&element.open_tag.name);
        self.node_name(&element.open_tag.name);

        self.attributes(element.attributes());

        if is_void {
            self.printer.word("/>");
        } else {
            self.printer.word(">")
        }
        self.visit_span(&element.open_tag.end_tag);
    }

    fn closing_tag(&mut self, element: &NodeElement) {
        self.visit_span(&element.close_tag);
        self.printer.word("</");
        self.node_name(element.name());
        self.printer.word(">");
    }

    fn attributes(&mut self, attributes: &[NodeAttribute]) {
        if attributes.is_empty() {
            return;
        }

        if let [attribute] = attributes {
            self.printer.cbox(0);
            self.printer.nbsp();
            self.visit_span(attribute);
            self.attribute(attribute);
            self.printer.end();
        } else {
            self.printer.cbox_indent();
            self.printer.space();

            let mut iter = attributes.iter().peekable();
            while let Some(attr) = iter.next() {
                self.visit_span(attr);
                self.attribute(attr);

                if iter.peek().is_some() {
                    self.printer.space()
                }
            }

            self.printer.zerobreak();
            self.printer.end_dedent();
        }
    }

    pub fn children(&mut self, children: &Vec<Node>, attribute_count: usize) {
        if children.is_empty() {
            return;
        }

        let is_textual = children
            .first()
            .map(|n| matches!(n, Node::Text(_) | Node::Block(_)))
            .unwrap_or_default();

        let soft_break = is_textual && attribute_count <= 1;

        if soft_break {
            self.printer.cbox_indent();
            self.printer.zerobreak();
            self.printer.ibox(0);
        } else {
            self.printer.neverbreak();
            self.printer.cbox_indent();
            self.printer.hardbreak();
        }

        let mut iter = children.iter().peekable();
        while let Some(child) = iter.next() {
            self.node(child);

            if iter.peek().is_some() {
                self.printer.space()
            }
        }

        if soft_break {
            self.printer.end();
            self.printer.zerobreak();
        } else {
            self.printer.hardbreak();
        }

        self.printer.end_dedent();
    }
}

fn is_void_element(name: &str, has_children: bool) -> bool {
    if name.chars().next().unwrap().is_uppercase() {
        !has_children
    } else {
        matches!(
            name,
            "area"
                | "base"
                | "br"
                | "col"
                | "embed"
                | "hr"
                | "img"
                | "input"
                | "link"
                | "meta"
                | "param"
                | "source"
                | "track"
                | "wbr"
        )
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::{
        formatter::FormatterSettings,
        test_helpers::{element, element_from_string, format_with, format_with_source},
    };

    macro_rules! format_element {
        ($($tt:tt)*) => {{
            let element = element! { $($tt)* };
            format_with(FormatterSettings { max_width: 40, ..Default::default() }, |formatter| {
                formatter.element(&element)
            })
        }};
    }
    macro_rules! format_element_from_string {
        ($val:expr) => {{
            let element = element_from_string! { $val };

            format_with_source(
                FormatterSettings {
                    max_width: 40,
                    ..Default::default()
                },
                $val,
                |formatter| formatter.element(&element),
            )
        }};
    }

    #[test]
    fn no_children() {
        let formatted = format_element! { < div > < / div > };
        insta::assert_snapshot!(formatted, @"<div></div>");
    }

    #[test]
    fn no_children_single_attr() {
        let formatted = format_element! { < div width=12 > < / div > };
        insta::assert_snapshot!(formatted, @"<div width=12></div>");
    }

    #[test]
    fn no_children_multi_attr() {
        let formatted = format_element! { <div key=23 width=100></div> };
        insta::assert_snapshot!(formatted, @"<div key=23 width=100></div>");
    }

    #[test]
    fn no_children_single_long_attr() {
        let formatted =
            format_element! { <div key=a::very::deeply::nested::module::generate_key()></div> };

        insta::assert_snapshot!(formatted, @"<div key=a::very::deeply::nested::module::generate_key()></div>");
    }

    #[test]
    fn no_children_multi_long_attr() {
        let formatted = format_element! { <div key=a::very::deeply::nested::module::generate_key() width=100></div> };
        insta::assert_snapshot!(formatted, @r###"
        <div
            key=a::very::deeply::nested::module::generate_key()
            width=100
        ></div>
        "###);
    }

    #[test]
    fn no_children_multi_attr_with_comment() {
        let formatted = format_element_from_string!(indoc! {"
        <div key=a
            // width
            width=100></div> 
        "});

        insta::assert_snapshot!(formatted, @r###"
        <div
            key=a
            // width
            width=100
        ></div>
        "###);
    }

    #[test]
    fn child_element() {
        let formatted = format_element! { <div><span>"hello"</span></div> };
        insta::assert_snapshot!(formatted, @r###"
        <div>
            <span>"hello"</span>
        </div>
        "###);
    }

    #[test]
    fn child_element_single_textual() {
        let formatted = format_element! { <div>"hello"</div> };
        insta::assert_snapshot!(formatted, @r###"<div>"hello"</div>"###);
    }

    #[test]
    fn child_element_single_textual_single_attr() {
        let formatted = format_element! { <div key=12>"hello"</div> };
        insta::assert_snapshot!(formatted, @r###"<div key=12>"hello"</div>"###);
    }

    #[test]
    fn child_element_single_textual_multi_attr() {
        let formatted = format_element! { <div key=12 width=100>"hello"</div> };
        insta::assert_snapshot!(formatted, @r###"
        <div key=12 width=100>
            "hello"
        </div>
        "###);
    }

    #[test]
    fn child_element_two_textual() {
        let formatted = format_element! { <div>"The count is" {count}</div> };
        insta::assert_snapshot!(formatted, @r###"<div>"The count is" {count}</div>"###);
    }

    #[test]
    fn child_element_many_textual() {
        let formatted = format_element! { <div>"The current count is: " {count} ". Increment by one is this: " {count + 1}</div> };
        insta::assert_snapshot!(formatted, @r###"
        <div>
            "The current count is: " {count}
            ". Increment by one is this: " {count + 1}
        </div>
        "###);
    }

    #[test]
    fn html_unquoted_text() {
        let formatted = format_element_from_string!(r##"<div>Unquoted text</div>"##);
        insta::assert_snapshot!(formatted, @r#"
        <div>
            Unquoted text
        </div>"#);
    }

    #[test]
    fn html_unquoted_text_with_surrounding_spaces() {
        let formatted = format_element_from_string!(r##"<div> Unquoted text with  spaces </div>"##);
        insta::assert_snapshot!(formatted, @r#"
        <div>
            Unquoted text with  spaces
        </div>"#);
    }

    #[test]
    fn html_unquoted_text_multiline() {
        let formatted = format_element_from_string!(indoc! {"
            <div>
            Unquoted text
                    with  spaces 
            </div>
        "});

        insta::assert_snapshot!(formatted, @r###"
        <div>
            Unquoted text
                    with  spaces
        </div>"###);
    }

    #[test]
    fn single_empty_line() {
        let formatted = format_element_from_string!(indoc! {r#"
            <div>
                <Nav/>

                <Main/>
            </div>
        "#});

        insta::assert_snapshot!(formatted, @r###"
        <div>
            <Nav/>
            <Main/>
        </div>
        "###);
    }

    #[test]
    fn multiple_empty_lines() {
        let formatted = format_element_from_string!(indoc! {r#"
            <div>
                <Nav/>



                <Main/>
            </div>
        "#});

        insta::assert_snapshot!(formatted, @r###"
        <div>
            <Nav/>
            <Main/>
        </div>
        "###);
    }

    #[test]
    fn surrounded_by_empty_lines() {
        let formatted = format_element_from_string!(indoc! {r#"

            <div>
                <Nav/>
                <Main/>
            </div>

        "#});

        insta::assert_snapshot!(formatted, @r###"
        <div>
            <Nav/>
            <Main/>
        </div>
        "###);
    }

    #[test]
    fn other_test() {
        let formatted = format_element_from_string!(indoc! {r#"
            <div>
                <div
                    class="foo"
                >
                    <i class="bi-google"></i>
                    "Sign in with google"
                </div>
            </div>
        "#});

        insta::assert_snapshot!(formatted, @r###"
        <div>
            <div class="foo">
                <i class="bi-google"></i>
                "Sign in with google"
            </div>
        </div>
        "###);
    }
}
