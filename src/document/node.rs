use std::{borrow::Cow, fmt::Debug};
#[cfg(feature = "document-breakdown")]
mod breakdown;
#[cfg(feature = "document-breakdown")]
pub use breakdown::*;

use crate::{element::ElementDebug, Document, Element};

/// Represents an XML node.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// XML Element
    Element(Element),
    /// XML Character Data ([specification](https://www.w3.org/TR/xml/#syntax))
    Text(String),
    /// Comments ([specification](https://www.w3.org/TR/xml/#sec-comments))
    Comment(String),
    /// CDATA ([specification](https://www.w3.org/TR/xml/#sec-cdata-sect))
    CData(String),
    /// Processing Instruction ([specification](https://www.w3.org/TR/xml/#sec-pi))
    PI(String),
    /// Document Type Declaration ([specification](https://www.w3.org/TR/xml/#sec-prolog-dtd))
    DocType(String),
}
impl From<Element> for Node {
    fn from(elem: Element) -> Self {
        Node::Element(elem)
    }
}
impl From<&Element> for Node {
    fn from(elem: &Element) -> Self {
        Node::Element(*elem)
    }
}
macro_rules! enum_is {
    [
        $(
            $(#[$docs:meta])*
            $fn_name:ident => $name:ident
        ),*
     ] => {
        $(
            $(#[$docs])*
            pub fn $fn_name(&self) -> bool {
                matches!(self, Node::$name(_))
            }
        )*
    };
}
impl Node {
    /// Useful to use inside `filter_map`.
    ///
    /// ```
    /// use edit_xml::{Document, Element};
    ///
    /// let mut doc = Document::parse_str(r#"<?xml version="1.0" encoding="UTF-8"?>
    /// <config>
    ///     Random Text
    ///     <max>1</max>
    /// </config>
    /// "#).unwrap();
    ///
    /// let elems: Vec<Element> = doc
    ///     .root_element()
    ///     .unwrap()
    ///     .children(&doc)
    ///     .iter()
    ///     .filter_map(|n| n.as_element())
    ///     .collect();
    /// ```
    pub fn as_element(&self) -> Option<Element> {
        match self {
            Self::Element(elem) => Some(*elem),
            _ => None,
        }
    }
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub(crate) fn build_text_content<'a>(&self, doc: &'a Document, buf: &'a mut String) {
        match self {
            Node::Element(elem) => elem.build_text_content(doc, buf),
            Node::Text(text) => buf.push_str(text),
            Node::CData(text) => buf.push_str(text),
            Node::PI(text) => buf.push_str(text),
            _ => {}
        }
    }

    /// Returns content if node is `Text`, `CData`, or `PI`.
    /// If node is `Element`, return [Element::text_content()]
    ///
    /// Implementation of [Node.textContent](https://developer.mozilla.org/en-US/docs/Web/API/Node/textContent)
    pub fn text_content(&self, doc: &Document) -> String {
        let mut buf = String::new();
        self.build_text_content(doc, &mut buf);
        buf
    }
    /// Returns content if node is `Text`, `CData`, or `PI`.
    ///
    /// If node is `Element` Cow will be owned.
    /// Otherwise, Cow will be borrowed.
    ///
    /// If None is returned it is a comment or doctype
    pub fn possible_borrowed_text(&self) -> Option<Cow<'_, str>> {
        match self {
            Node::Text(text) => Some(Cow::Borrowed(text)),
            Node::CData(text) => Some(Cow::Borrowed(text)),
            Node::PI(text) => Some(Cow::Borrowed(text)),
            Node::Element(element) => Some(Cow::Owned(element.text_content(&Document::new()))),
            _ => None,
        }
    }
    /// Debug the node
    pub fn debug<'node, 'doc>(&'node self, doc: &'doc Document) -> NodeDebug<'node, 'doc> {
        NodeDebug::new(self, doc)
    }
    enum_is![
        /// Returns true if the node is a text node
        /// ```
        /// use edit_xml::Node;
        /// let node = Node::Text("Hello".to_string());
        /// assert!(node.is_text());
        /// ```
        is_text => Text,
        /// Returns true if the node is a comment node
        /// ```
        /// use edit_xml::Node;
        /// let node = Node::Comment("Hello".to_string());
        /// assert!(node.is_comment());
        /// ```
        is_comment => Comment,
        /// Returns true if the node is a CDATA node
        /// ```
        /// use edit_xml::Node;
        /// let node = Node::CData("Hello".to_string());
        /// assert!(node.is_cdata());
        /// ```
        is_cdata => CData,
        /// Returns true if the node is a Processing Instruction node
        /// ```
        /// use edit_xml::Node;
        /// let node = Node::PI("Hello".to_string());
        /// assert!(node.is_pi());
        /// ```
        is_pi => PI,
        /// Returns true if the node is a Document Type Declaration
        /// ```
        /// use edit_xml::Node;
        /// let node = Node::DocType("Hello".to_string());
        /// assert!(node.is_doctype());
        /// ```
        is_doctype => DocType,
        /// Returns true if the node is an element
        is_element => Element
    ];
}
pub enum NodeDebug<'node, 'doc> {
    /// Uses ElementDebug to debug the element
    Element(ElementDebug<'node, 'doc>),
    /// Text node
    Text(&'node str),
    /// Comment node
    Comment(&'node str),
    /// CDATA node
    CData(&'node str),
    /// Processing Instruction node
    PI(&'node str),
    /// Document Type Declaration node
    DocType(&'node str),
}
impl<'node, 'doc> NodeDebug<'node, 'doc> {
    pub fn new(node: &'node Node, doc: &'doc Document) -> Self {
        match node {
            Node::Element(elem) => NodeDebug::Element(ElementDebug::new(elem, doc)),
            Node::Text(text) => NodeDebug::Text(text),
            Node::Comment(text) => NodeDebug::Comment(text),
            Node::CData(text) => NodeDebug::CData(text),
            Node::PI(text) => NodeDebug::PI(text),
            Node::DocType(text) => NodeDebug::DocType(text),
        }
    }
}

impl Debug for NodeDebug<'_, '_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            NodeDebug::Element(element) => element.fmt(f),
            NodeDebug::Text(text_content) => Debug::fmt(text_content, f),
            NodeDebug::Comment(f0) => f.debug_tuple("Comment").field(&f0).finish(),
            NodeDebug::CData(f0) => f.debug_tuple("CData").field(&f0).finish(),
            NodeDebug::PI(f0) => f.debug_tuple("PI").field(&f0).finish(),
            NodeDebug::DocType(f0) => f.debug_tuple("DocType").field(&f0).finish(),
        }
    }
}
