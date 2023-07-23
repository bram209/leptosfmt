use rstml::node::NodeFragment;

use crate::formatter::Formatter;

impl Formatter<'_> {
    pub fn fragment(&mut self, fragment: &NodeFragment) {
        self.visit_spanned(&fragment.tag_open);
        self.printer.word("<>");
        self.children(&fragment.children, 0);
        self.printer.word("</>");
        self.visit_spanned(&fragment.tag_close);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        formatter::FormatterSettings,
        test_helpers::{format_with, fragment},
    };

    macro_rules! format_fragment {
        ($($tt:tt)*) => {{
            let fragment = fragment! { $($tt)* };
            let settings = FormatterSettings { max_width: 40, ..Default::default() };
            format_with(settings, |formatter| {
                formatter.fragment(&fragment);
            })
        }};
    }

    #[test]
    fn fragment_no_children() {
        let formatted = format_fragment! { <> </> };
        insta::assert_snapshot!(formatted, @"<></>");
    }

    #[test]
    fn fragment_child_element() {
        let formatted = format_fragment! { <><span>"hello"</span></> };
        insta::assert_snapshot!(formatted, @r###"
        <>
            <span>"hello"</span>
        </>
        "###);
    }

    #[test]
    fn fragment_child_element_single_textual() {
        let formatted = format_fragment! { <>"hello"</> };
        insta::assert_snapshot!(formatted, @r###"<>"hello"</>"###);
    }

    #[test]
    fn fragment_child_element_two_textual() {
        let formatted = format_fragment! { <>"The count is" {count}</> };
        insta::assert_snapshot!(formatted, @r###"<>"The count is" {count}</>"###);
    }

    #[test]
    fn fragment_child_element_many_textual() {
        let formatted = format_fragment! { <>"The current count is: " {count} ". Increment by one is this: " {count + 1}</> };
        insta::assert_snapshot!(formatted, @r###"
        <>
            "The current count is: " {count}
            ". Increment by one is this: " {count + 1}
        </>
        "###);
    }
}
