use syn_rsx::{Node, NodeAttribute, NodeElement, NodeFragment};

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

pub(crate) use attribute;
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
