use syn_rsx::{Node, NodeBlock, NodeComment, NodeDoctype, NodeName, NodeText, NodeValueExpr};

use crate::{formatter::Formatter, source_file::format_expr_source};

impl Formatter {
    pub fn node(&mut self, node: &Node) {
        match node {
            Node::Element(ele) => self.element(ele),
            Node::Attribute(attr) => self.attribute(attr),
            Node::Text(text) => self.node_text(text),
            Node::Comment(comment) => self.comment(comment),
            Node::Doctype(doctype) => self.doctype(doctype),
            Node::Block(block) => self.node_block(block),
            Node::Fragment(frag) => self.fragment(frag),
        }
    }

    pub fn comment(&mut self, comment: &NodeComment) {
        self.printer.word("<!-- ");
        self.node_value(&comment.value);
        self.printer.word(" -->");
    }

    pub fn doctype(&mut self, doctype: &NodeDoctype) {
        self.node_value(&doctype.value);
    }

    pub fn node_text(&mut self, text: &NodeText) {
        self.node_value(&text.value);
    }

    pub fn node_name(&mut self, name: &NodeName) {
        self.printer.word(name.to_string());
    }

    pub fn node_block(&mut self, block: &NodeBlock) {
        self.node_value(&block.value)
    }

    pub fn node_value(&mut self, value: &NodeValueExpr) {
        // if single line expression, format as '{expr}' instead of '{ expr }' (prettyplease inserts a space)
        if let syn::Expr::Block(expr_block) = value.as_ref() {
            if expr_block.attrs.is_empty() {
                if let [syn::Stmt::Expr(single_expr)] = &expr_block.block.stmts[..] {
                    // wrap with braces and do NOT insert spaces
                    self.printer.word("{");
                    self.expr(single_expr);
                    self.printer.word("}");
                    return;
                }
            }
        }

        self.expr(value.as_ref())
    }

    fn expr(&mut self, expr: &syn::Expr) {
        let formatted = leptosfmt_prettyplease::unparse_expr(expr);
        let formatted = format_expr_source(&formatted, self.settings).unwrap_or(formatted);

        let mut iter = formatted.lines().peekable();
        while let Some(line) = iter.next() {
            self.printer.word(line.to_owned());

            if iter.peek().is_some() {
                self.printer.hardbreak();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::formatter::*;
    use crate::test_helpers::comment;

    macro_rules! format_comment {
        ($($tt:tt)*) => {{
            let comment = comment! { $($tt)* };
            let mut formatter = Formatter::new(FormatterSettings { max_width: 40, ..Default::default() });
            formatter.comment(&comment);
            formatter.printer.eof()
        }};
    }

    #[test]
    fn html_comment() {
        let formatted = format_comment!(<!--   "comment"   -->);
        insta::assert_snapshot!(formatted, @r###"<!-- "comment" -->"###);
    }

    #[test]
    fn html_comment_long() {
        let formatted = format_comment!(<!--   "this is a very loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong comment"   -->);
        insta::assert_snapshot!(formatted, @r###"<!-- "this is a very loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong comment" -->"###);
    }
}
