use rstml::node::{Node, NodeBlock, NodeComment, NodeDoctype, NodeName, NodeText, RawText};
use syn::spanned::Spanned;

use crate::formatter::Formatter;

impl Formatter<'_> {
    pub fn node(&mut self, node: &Node) {
        // self.write_comments(node.span().start().line - 1);
        match node {
            Node::Element(_) | Node::Fragment(_) => {}
            _ => self.visit_span(node),
        }

        match node {
            Node::Element(ele) => self.element(ele),
            Node::Fragment(frag) => self.fragment(frag),
            Node::Text(text) => self.node_text(text),
            Node::RawText(text) => self.raw_text(text, true),
            Node::Comment(comment) => self.comment(comment),
            Node::Doctype(doctype) => self.doctype(doctype),
            Node::Block(block) => self.node_block(block),
        };
    }

    pub fn comment(&mut self, comment: &NodeComment) {
        self.printer.word("<!-- ");
        self.literal_str(&comment.value);
        self.printer.word(" -->");
    }

    pub fn doctype(&mut self, doctype: &NodeDoctype) {
        self.printer.word("<!DOCTYPE ");
        self.raw_text(&doctype.value, false);
        self.printer.word("> ");
    }

    pub fn node_text(&mut self, text: &NodeText) {
        self.literal_str(&text.value);
    }

    pub fn raw_text(&mut self, raw_text: &RawText, use_source_text: bool) {
        let text = if use_source_text {
            raw_text.to_source_text(false)
                .expect("Cannot format unquoted text, no source text available, or unquoted text is used outside of element.")
        } else {
            raw_text.to_token_stream_string()
        };

        self.string(&text, raw_text.span().start().column);
    }

    pub fn node_name(&mut self, name: &NodeName) {
        self.printer.word(name.to_string());
    }

    pub fn node_block(&mut self, block: &NodeBlock) {
        match block {
            NodeBlock::Invalid { .. } => panic!("Invalid block will not pass cargo check"), // but we can keep them instead of panic
            NodeBlock::ValidBlock(b) => self.node_value_block_expr(b, false, false),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::formatter::*;
    use crate::test_helpers::{comment, doctype, format_with};

    macro_rules! format_comment {
        ($($tt:tt)*) => {{
            let comment = comment! { $($tt)* };
            let settings = FormatterSettings { max_width: 40, ..Default::default() };
            format_with(settings, |formatter| {
                formatter.comment(&comment);
            })
        }};
    }

    macro_rules! format_doctype {
        ($($tt:tt)*) => {{
            let doctype = doctype! { $($tt)* };
            let settings = FormatterSettings { max_width: 40, ..Default::default() };
            format_with(settings, |formatter| {
                formatter.doctype(&doctype);
            })
        }};
    }

    #[test]
    fn html_comment() {
        let formatted = format_comment!(<!--   "comment"   -->);
        insta::assert_snapshot!(formatted, @r#"<!-- "comment" -->"#);
    }

    #[test]
    fn html_comment_long() {
        let formatted = format_comment!(<!--   "this is a very loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong comment"   -->);
        insta::assert_snapshot!(formatted, @r#"<!-- "this is a very loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong comment" -->"#);
    }

    #[test]
    fn html_doctype() {
        let formatted = format_doctype!(< !DOCTYPE html   >);
        insta::assert_snapshot!(formatted, @"<!DOCTYPE html> ");
    }
}
