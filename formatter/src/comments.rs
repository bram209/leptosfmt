use crop::{Rope, RopeSlice};
use proc_macro2::{LineColumn, Span};
use rstml::node::{Node, NodeElement, NodeFragment};
use syn::{bracketed, spanned::Spanned};

use crate::line_column_to_byte;

pub fn collect_comments(nodes: Vec<Node>) {}

struct NonDocComment {
    comment: String,
    line_index: usize,
}

struct CommentVisitor<'a> {
    source: &'a Rope,
    last_span: Option<Span>,
    comments: Vec<NonDocComment>,
}

impl CommentVisitor<'_> {
    fn visit_span(&mut self, spanned: impl Spanned) {
        let span = spanned.span();

        if let Some(last_span) = self.last_span {
            if last_span.end().line != span.start().line {
                let text = get_text_beween_spans(self.source, last_span.end(), span.start());
                for (idx, line) in text.lines().enumerate() {
                    let line = line.to_string();
                    let Some(comment) = line.split("//").nth(1).map(|l| NonDocComment {
                        line_index: last_span.end().line - 1 + idx,
                        comment: l.to_owned(),
                    }) else { continue; };

                    self.comments.push(comment)
                }
            }
        }

        self.last_span = Some(span);
    }

    fn visit_node(&mut self, node: &Node) {
        match node {
            Node::Fragment(f) => self.visit_fragment(f),
            Node::Element(e) => self.visit_element(e),
            _ => self.visit_span(node),
        }
    }

    fn visit_fragment(&mut self, fragment: &NodeFragment) {
        self.visit_span(&fragment.tag_open);
        for child in fragment.children.iter() {
            self.visit_node(child);
        }
        self.visit_span(&fragment.tag_close);
    }

    fn visit_element(&mut self, element: &NodeElement) {
        self.visit_span(&element.open_tag);
        for child in element.children.iter() {
            self.visit_node(child);
        }
        self.visit_span(&element.close_tag);
    }
}

fn get_text_beween_spans(rope: &Rope, start: LineColumn, end: LineColumn) -> RopeSlice<'_> {
    let start_byte = line_column_to_byte(&rope, start);
    let end_byte = line_column_to_byte(&rope, end);

    return rope.byte_slice(start_byte..end_byte);
}
