use syn_rsx::{Node, NodeAttribute, NodeComment, NodeDoctype, NodeElement, NodeFragment};

macro_rules! attribute {
    ($($tt:tt)*) => {
        {
        let tokens = quote::quote! { <tag $($tt)* /> };
        let nodes = syn_rsx::parse2(tokens).unwrap();
        crate::test_helpers::get_element_attribute(nodes, 0, 0)
    }};
}

macro_rules! element {
    ($($tt:tt)*) => {
        {
        let tokens = quote::quote! { $($tt)* };
        let nodes = syn_rsx::parse2(tokens).unwrap();
        crate::test_helpers::get_element(nodes, 0)
    }};
}

macro_rules! fragment {
    ($($tt:tt)*) => {
        {
        let tokens = quote::quote! { $($tt)* };
        let nodes = syn_rsx::parse2(tokens).unwrap();
        crate::test_helpers::get_fragment(nodes, 0)
    }};
}

macro_rules! comment {
    ($($tt:tt)*) => {
        {
        let tokens = quote::quote! { $($tt)* };
        let nodes = syn_rsx::parse2(tokens).unwrap();
        crate::test_helpers::get_comment(nodes, 0)
    }};
}

macro_rules! doctype {
    ($($tt:tt)*) => {
        {
        let tokens = quote::quote! { $($tt)* };
        let nodes = syn_rsx::parse2(tokens).unwrap();
        crate::test_helpers::get_doctype(nodes, 0)
    }};
}

pub(crate) use attribute;
pub(crate) use comment;
pub(crate) use doctype;
pub(crate) use element;
pub(crate) use fragment;

pub fn get_element_attribute(
    mut nodes: Vec<Node>,
    element_index: usize,
    attribute_index: usize,
) -> NodeAttribute {
    let Node::Element(mut element) =
        nodes.swap_remove(element_index) else { panic!("expected element") };
    let Node::Attribute(attribute) =
        element.attributes.swap_remove(attribute_index) else { panic!("expected attribute") };

    attribute
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
