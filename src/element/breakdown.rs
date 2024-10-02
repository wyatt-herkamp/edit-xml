use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::utils::HashMap;

use crate::{Document, Element, NodeBreakdown};
impl Element {
    /// Create a breakdown of the element.
    ///
    /// This will clone all the data that belongs to the element and its children into a single object.
    ///
    /// This is useful for debugging and serializing the element.
    ///
    /// All the data is cloned. This is not a memory efficient or fast way to access the data.
    /// # Note
    /// The data structure is not stable and may change in the future.
    #[instrument]
    pub fn breakdown(&self, doc: &Document) -> ElementBreakdown {
        ElementBreakdown::new(*self, doc)
    }
}
/// Loops through all the children of an element and creates a breakdown of each child.
#[instrument]
fn get_children(element: Element, doc: &Document) -> Vec<NodeBreakdown> {
    element
        .children(doc)
        .iter()
        .map(|child| {
            let node = child.clone();
            NodeBreakdown::new(node, doc)
        })
        .collect()
}
/// A breakdown of an element. This will clone all the data that belongs to the element and its children into a single object.
///
/// This is useful for debugging and serializing the element.
///
/// All the data is cloned. This is not a memory efficient or fast way to access the data.
/// # Note
/// The data structure is not stable and may change in the future.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElementBreakdown {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub namespace_decls: HashMap<String, String>,
    pub children: Vec<NodeBreakdown>,
    pub is_root_element: bool,
    // TODO: Can we just use a reference to the document? and manually implement Serialize/Deserialize?
}
impl ElementBreakdown {
    #[instrument]
    pub fn new(element: Element, doc: &Document) -> Self {
        let name = element.name(doc).to_owned();
        let attributes = element.attributes(doc).clone();
        let namespace_decls = element.namespace_decls(doc).clone();
        let children = get_children(element, doc);
        Self {
            name,
            attributes,
            namespace_decls,
            children,
            is_root_element: element.is_root(doc),
        }
    }
}
