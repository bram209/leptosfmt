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

pub fn format_with(settings: FormatterSettings, run: impl FnOnce(&mut Formatter)) -> String {
    let mut printer = Printer::new((&settings).into());
    let mut formatter = Formatter::new(settings, &mut printer);
    run(&mut formatter);
    printer.eof()
}
