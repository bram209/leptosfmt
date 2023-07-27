use std::str::FromStr;

use crop::Rope;
use leptosfmt_pretty_printer::Printer;
use rstml::node::{Node, NodeAttribute, NodeComment, NodeDoctype, NodeElement, NodeFragment};

macro_rules! attribute {
    ($($tt:tt)*) => {
        {
        let tokens = quote::quote! { <tag $($tt)* /> };
        let nodes = rstml::parse2(tokens).unwrap();
        crate::test_helpers::get_element_attribute(nodes, 0, 0)
    }};
}

macro_rules! element {
    ($($tt:tt)*) => {
        {
        let tokens = quote::quote! { $($tt)* };
        let nodes = rstml::parse2(tokens).unwrap();
        crate::test_helpers::get_element(nodes, 0)
    }};
}

// Same as element, but use string representation of token stream.
// This is usefull when testing unquoted text,
// because current `quote!` implementation cannot provide `Span::source_text`
// that is used in `raw_text` handler
macro_rules! element_from_string {
    ($val: expr) => {{
        let tokens = <proc_macro2::TokenStream as std::str::FromStr>::from_str($val).unwrap();
        let nodes = rstml::parse2(tokens).unwrap();
        crate::test_helpers::get_element(nodes, 0)
    }};
}

macro_rules! fragment {
    ($($tt:tt)*) => {
        {
        let tokens = quote::quote! { $($tt)* };
        let nodes = rstml::parse2(tokens).unwrap();
        crate::test_helpers::get_fragment(nodes, 0)
    }};
}

macro_rules! comment {
    ($($tt:tt)*) => {
        {
        let tokens = quote::quote! { $($tt)* };
        let nodes = rstml::parse2(tokens).unwrap();
        crate::test_helpers::get_comment(nodes, 0)
    }};
}

macro_rules! doctype {
    ($($tt:tt)*) => {
        {
        let tokens = quote::quote! { $($tt)* };
        let nodes = rstml::parse2(tokens).unwrap();
        crate::test_helpers::get_doctype(nodes, 0)
    }};
}

pub(crate) use attribute;
pub(crate) use comment;
pub(crate) use doctype;
pub(crate) use element;
pub(crate) use element_from_string;
pub(crate) use fragment;

use crate::{Formatter, FormatterSettings};

pub fn get_element_attribute(
    mut nodes: Vec<Node>,
    element_index: usize,
    attribute_index: usize,
) -> NodeAttribute {
    let Node::Element(element) =
        nodes.swap_remove(element_index) else { panic!("expected element") };
    element
        .attributes()
        .get(attribute_index)
        .expect("attribute exist")
        .clone()
}

pub fn get_element(mut nodes: Vec<Node>, element_index: usize) -> NodeElement {
    let Node::Element(element) = nodes.swap_remove(element_index) else { panic!("expected element") };
    element
}

pub fn get_fragment(mut nodes: Vec<Node>, fragment_index: usize) -> NodeFragment {
    let Node::Fragment(fragment) = nodes.swap_remove(fragment_index) else { panic!("expected fragment") };
    fragment
}

pub fn get_comment(mut nodes: Vec<Node>, comment_index: usize) -> NodeComment {
    let Node::Comment(comment) = nodes.swap_remove(comment_index) else { panic!("expected comment") };
    comment
}

pub fn get_doctype(mut nodes: Vec<Node>, doctype_index: usize) -> NodeDoctype {
    let Node::Doctype(doctype) = nodes.swap_remove(doctype_index) else { panic!("expected doctype") };
    doctype
}

pub fn format_with_source(
    settings: FormatterSettings,
    source: &str,
    run: impl FnOnce(&mut Formatter),
) -> String {
    let mut printer = Printer::new((&settings).into());
    let rope = Rope::from_str(source).unwrap();
    let tokens = <proc_macro2::TokenStream as std::str::FromStr>::from_str(source).unwrap();
    let whitespace = crate::collect_comments::extract_whitespace_and_comments(&rope, tokens);
    let mut formatter = Formatter::with_source(settings, &mut printer, &rope, whitespace);
    run(&mut formatter);
    printer.eof()
}

pub fn format_with(settings: FormatterSettings, run: impl FnOnce(&mut Formatter)) -> String {
    let mut printer = Printer::new((&settings).into());
    let mut formatter = Formatter::new(settings, &mut printer);
    run(&mut formatter);
    printer.eof()
}
