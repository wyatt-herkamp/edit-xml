use serde::{Deserialize, Serialize};

use crate::{Document, ElementBreakdown, Node};
impl Node {
    /// Create a breakdown of the node.
    ///
    /// This will clone all the data that belongs to the node and its children into a single object.
    ///
    /// This is useful for debugging and serializing the node.
    ///
    /// All the data is cloned. This is not a memory efficient or fast way to access the data.
    /// # Note
    /// The data structure is not stable and may change in the future.
    pub fn breakdown(&self, doc: &Document) -> NodeBreakdown {
        NodeBreakdown::new(self.clone(), doc)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum NodeBreakdown {
    Element(ElementBreakdown),
    Text(String),
    Comment(String),
    CData(String),
    PI(String),
    DocType(String),
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
                matches!(self, NodeBreakdown::$name(_))
            }
        )*
    };
}
impl NodeBreakdown {
    pub fn new(node: Node, doc: &Document) -> Self {
        match node {
            Node::Element(element) => NodeBreakdown::Element(ElementBreakdown::new(element, doc)),
            Node::Text(text) => NodeBreakdown::Text(text),
            Node::Comment(comment) => NodeBreakdown::Comment(comment),
            Node::CData(cdata) => NodeBreakdown::CData(cdata),
            Node::PI(pi) => NodeBreakdown::PI(pi),
            Node::DocType(doctype) => NodeBreakdown::DocType(doctype),
        }
    }
    enum_is![
        /// Check if the breakdown is an Element
        is_element => Element,
        /// Check if the breakdown is a Text node
        is_text => Text,
        /// Check if the breakdown is a Comment node
        is_comment => Comment,
        /// Check if the breakdown is a CData node
        is_cdata => CData,
        /// Check if the breakdown is a Processing Instruction node
        is_pi => PI,
        /// Check if the breakdown is a Document Type Declaration node
        is_doctype => DocType
    ];
}
