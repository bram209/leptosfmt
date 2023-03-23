use syn_rsx::NodeFragment;

use crate::formatter::Formatter;

impl<'a> Formatter<'a> {
    pub fn fragment(&mut self, fragment: &NodeFragment) {
        self.printer.word("<>");
        self.children(&fragment.children, 0);
        self.printer.word("</>");
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        formatter::{Formatter, FormatterSettings},
        test_helpers::fragment,
    };

    macro_rules! format_fragment {
        ($($tt:tt)*) => {{
            let fragment = fragment! { $($tt)* };
            let mut formatter = Formatter::new(FormatterSettings { max_width: 40, ..Default::default() });
            formatter.fragment(&fragment);
            formatter.printer.eof()
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
