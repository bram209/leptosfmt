use crate::{formatter::Formatter, ClosingTagStyle};

use rstml::node::{Node, NodeAttribute, NodeElement};
use syn::spanned::Spanned;

impl Formatter<'_> {
    pub fn element(&mut self, element: &NodeElement) {
        let name = element.name().to_string();
        let is_self_closing = is_self_closing(element, &name, self.settings.closing_tag_style);

        self.opening_tag(element, is_self_closing);

        if !is_self_closing {
            self.children(&element.children, element.attributes().len());
            self.flush_comments(element.close_tag.span().end().line - 1);
            self.closing_tag(element)
        }
    }

    fn opening_tag(&mut self, element: &NodeElement, is_self_closing: bool) {
        self.printer.word("<");
        self.node_name(&element.open_tag.name);

        self.attributes(element.attributes());

        if is_self_closing {
            self.printer.nbsp();
            self.printer.word("/>");
        } else {
            self.printer.word(">")
        }
    }

    fn closing_tag(&mut self, element: &NodeElement) {
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
            self.attribute(attribute);
            self.printer.end();
        } else {
            self.printer.cbox_indent();
            self.printer.space();

            let mut iter = attributes.iter().peekable();
            while let Some(attr) = iter.next() {
                self.attribute(attr);

                if iter.peek().is_some() {
                    self.printer.space()
                }
            }

            self.printer.zerobreak();
            self.printer.end_dedent();
        }
    }

    pub fn children(&mut self, children: &[Node], attribute_count: usize) {
        if children.is_empty() {
            return;
        }

        let is_textual = children
            .first()
            .map(|n| matches!(n, Node::Text(_) | Node::RawText(_) | Node::Block(_)))
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

            if let Some(next_child) = iter.peek() {
                let curr_end = child.span().end();
                let next_start = next_child.span().start();
                let consecutive =
                    curr_end.line == next_start.line && next_start.column == curr_end.column;

                if !matches!(next_child, Node::RawText(_)) && !consecutive {
                    self.printer.space()
                } else {
                    self.printer.zerobreak()
                }
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

fn is_void_element(name: &str) -> bool {
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

fn is_self_closing(element: &NodeElement, name: &str, closing_tag_style: ClosingTagStyle) -> bool {
    if !element.children.is_empty() {
        return false;
    }

    if is_void_element(name) {
        return true;
    };

    // At this point, it must be a non-void element that has no children
    match closing_tag_style {
        ClosingTagStyle::Preserve => element.close_tag.is_none(),
        ClosingTagStyle::SelfClosing => true,
        ClosingTagStyle::NonSelfClosing => false,
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::{
        formatter::ClosingTagStyle,
        formatter::FormatterSettings,
        test_helpers::{element, format_element_from_string, format_with},
    };

    macro_rules! format_element {
        ($($tt:tt)*) => {{
            format_element_with!(Default::default(), $($tt)*)
        }};
    }

    macro_rules! format_element_with_closing_style {
        ($style:expr, $($tt:tt)*) => {{
            format_element_with!(FormatterSettings {
                closing_tag_style: $style,
                ..Default::default()
            }, $($tt)*)
        }};
    }

    macro_rules! format_element_with {
        ($settings:expr, $($tt:tt)*) => {{
            let element = element! { $($tt)* };
            format_with(FormatterSettings { max_width: 40, ..$settings }, |formatter| {
                formatter.element(&element)
            })
        }};
    }

    macro_rules! format_element_from_string {
        ($val:expr) => {{
            format_element_from_string(
                FormatterSettings {
                    max_width: 40,
                    ..Default::default()
                },
                $val,
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
        insta::assert_snapshot!(formatted, @r#"
        <div>
            <span>"hello"</span>
        </div>
        "#);
    }

    #[test]
    fn child_element_single_textual() {
        let formatted = format_element! { <div>"hello"</div> };
        insta::assert_snapshot!(formatted, @r#"<div>"hello"</div>"#);
    }

    #[test]
    fn child_element_single_textual_unquoted() {
        let formatted = format_element_from_string!("<div>hello</div>");
        insta::assert_snapshot!(formatted, @r###"<div>hello</div>"###);
    }

    #[test]
    fn child_element_single_textual_single_attr() {
        let formatted = format_element! { <div key=12>"hello"</div> };
        insta::assert_snapshot!(formatted, @r#"<div key=12>"hello"</div>"#);
    }

    #[test]
    fn child_element_single_textual_multi_attr() {
        let formatted = format_element! { <div key=12 width=100>"hello"</div> };
        insta::assert_snapshot!(formatted, @r#"
        <div key=12 width=100>
            "hello"
        </div>
        "#);
    }

    #[test]
    fn child_element_two_textual() {
        let formatted = format_element! { <div>"The count is " {count}</div> };
        insta::assert_snapshot!(formatted, @r#"<div>"The count is " {count}</div>"#);
    }

    #[test]
    fn child_element_many_textual() {
        let formatted = format_element! { <div>"The current count is: " {count} ". Increment by one is this: " {count + 1}</div> };
        insta::assert_snapshot!(formatted, @r#"
        <div>
            "The current count is: " {count}
            ". Increment by one is this: " {count + 1}
        </div>
        "#);
    }

    #[test]
    fn child_element_two_textual_unquoted() {
        let formatted = format_element_from_string! { "<div>The count is {count}.</div>" };
        insta::assert_snapshot!(formatted, @r#"<div>The count is {count}.</div>"#);
    }

    #[test]
    fn child_element_two_textual_unquoted_no_trailingspace() {
        let formatted = format_element_from_string! { "<div>The count is{count}</div>" };
        insta::assert_snapshot!(formatted, @r#"<div>The count is{count}</div>"#);
    }

    #[test]
    fn child_element_many_textual_unquoted() {
        let formatted = format_element_from_string! { "<div>The current count is: {count}. Increment by one is this: {count + 1}</div>" };
        insta::assert_snapshot!(formatted, @r###"
        <div>
            The current count is: {count}. Increment by one is this:
            {count + 1}
        </div>
        "###);
    }
    // view! { <p>Something: {something} .</p> }

    #[test]
    fn html_unquoted_text() {
        let formatted = format_element_from_string!(r##"<div>Unquoted text</div>"##);
        insta::assert_snapshot!(formatted, @"<div>Unquoted text</div>");
    }

    #[test]
    fn html_unquoted_text_with_surrounding_spaces() {
        let formatted = format_element_from_string!(r##"<div> Unquoted text with  spaces </div>"##);
        insta::assert_snapshot!(formatted, @"<div>Unquoted text with  spaces</div>");
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
                <Nav />

                <Main />
            </div>
        "#});

        insta::assert_snapshot!(formatted, @r###"
        <div>
            <Nav />

            <Main />
        </div>
        "###);
    }

    #[test]
    fn multiple_empty_lines() {
        let formatted = format_element_from_string!(indoc! {r#"
            <div>
                <Nav />



                <Main />
            </div>
        "#});

        insta::assert_snapshot!(formatted, @r###"
        <div>
            <Nav />

            <Main />
        </div>
        "###);
    }

    #[test]
    fn surrounded_by_empty_lines() {
        let formatted = format_element_from_string!(indoc! {r#"

            <div>
                <Nav />
                <Main />
            </div>

        "#});

        insta::assert_snapshot!(formatted, @r###"
        <div>
            <Nav />
            <Main />
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

        insta::assert_snapshot!(formatted, @r#"
        <div>
            <div class="foo">
                <i class="bi-google"></i>
                "Sign in with google"
            </div>
        </div>
        "#);
    }

    // Closing Tags Behaviour

    #[test]
    fn void_element_no_children_separate_closing_tag() {
        let preserve_formatted =
            format_element_with_closing_style! { ClosingTagStyle::Preserve, < input >< / input > };
        let self_closing_formatted = format_element_with_closing_style! { ClosingTagStyle::SelfClosing, < input >< / input > };
        let non_self_closing_formatted = format_element_with_closing_style! { ClosingTagStyle::NonSelfClosing, < input >< / input > };

        insta::assert_snapshot!(preserve_formatted, @"<input />");
        insta::assert_snapshot!(self_closing_formatted, @"<input />");
        insta::assert_snapshot!(non_self_closing_formatted, @"<input />");
    }

    #[test]
    fn void_element_no_children_self_closing_tag_one_line() {
        let preserve_formatted =
            format_element_with_closing_style! { ClosingTagStyle::Preserve,  < input / > };
        let self_closing_formatted =
            format_element_with_closing_style! { ClosingTagStyle::SelfClosing,  < input / > };
        let non_self_closing_formatted =
            format_element_with_closing_style! { ClosingTagStyle::NonSelfClosing,  < input / > };

        insta::assert_snapshot!(preserve_formatted, @"<input />");
        insta::assert_snapshot!(self_closing_formatted, @"<input />");
        insta::assert_snapshot!(non_self_closing_formatted, @"<input />");
    }

    #[test]
    fn void_element_no_children_self_closing_tag_multi_line() {
        let preserve_formatted = format_element_with_closing_style! { ClosingTagStyle::Preserve,  < input key=1 class="veryveryvery longlonglong attributesattributesattributes listlistlist" / > };
        let self_closing_formatted = format_element_with_closing_style! { ClosingTagStyle::SelfClosing,  < input key=1 class="veryveryvery longlonglong attributesattributesattributes listlistlist" / > };
        let non_self_closing_formatted = format_element_with_closing_style! { ClosingTagStyle::NonSelfClosing,  < input key=1 class="veryveryvery longlonglong attributesattributesattributes listlistlist" / > };

        insta::assert_snapshot!(preserve_formatted, @r#"
        <input
            key=1
            class="veryveryvery longlonglong attributesattributesattributes listlistlist"
        />
        "#);
        insta::assert_snapshot!(self_closing_formatted, @r#"
        <input
            key=1
            class="veryveryvery longlonglong attributesattributesattributes listlistlist"
        />
        "#);
        insta::assert_snapshot!(non_self_closing_formatted, @r#"
        <input
            key=1
            class="veryveryvery longlonglong attributesattributesattributes listlistlist"
        />
        "#);
    }

    #[test]
    fn non_void_element_with_child() {
        let preserve_formatted = format_element_with_closing_style! { ClosingTagStyle::Preserve,  < div > "Child" < / div > };
        let self_closing_formatted = format_element_with_closing_style! { ClosingTagStyle::SelfClosing,  < div > "Child" < / div > };
        let non_self_closing_formatted = format_element_with_closing_style! { ClosingTagStyle::NonSelfClosing,  < div > "Child" < / div > };

        insta::assert_snapshot!(preserve_formatted, @r#"<div>"Child"</div>"#);
        insta::assert_snapshot!(self_closing_formatted, @r#"<div>"Child"</div>"#);
        insta::assert_snapshot!(non_self_closing_formatted, @r#"<div>"Child"</div>"#);
    }

    #[test]
    fn non_void_element_no_children_separate_closing_tag() {
        let preserve_formatted =
            format_element_with_closing_style! { ClosingTagStyle::Preserve,  < div >< / div > };
        let self_closing_formatted =
            format_element_with_closing_style! { ClosingTagStyle::SelfClosing,  < div >< / div > };
        let non_self_closing_formatted = format_element_with_closing_style! { ClosingTagStyle::NonSelfClosing,  < div >< / div > };

        insta::assert_snapshot!(preserve_formatted, @"<div></div>");
        insta::assert_snapshot!(self_closing_formatted, @"<div />");
        insta::assert_snapshot!(non_self_closing_formatted, @"<div></div>");
    }

    #[test]
    fn non_void_element_no_children_self_closing_tag() {
        let preserve_formatted =
            format_element_with_closing_style! { ClosingTagStyle::Preserve,  < div / > };
        let self_closing_formatted =
            format_element_with_closing_style! { ClosingTagStyle::SelfClosing,  < div / > };
        let non_self_closing_formatted =
            format_element_with_closing_style! { ClosingTagStyle::NonSelfClosing,  < div / > };

        insta::assert_snapshot!(preserve_formatted, @"<div />");
        insta::assert_snapshot!(self_closing_formatted, @"<div />");
        insta::assert_snapshot!(non_self_closing_formatted, @"<div></div>");
    }
}
