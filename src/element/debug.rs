use std::fmt::Debug;

use crate::Document;

use super::Element;

/// Debug implementation for Element
///
/// This is a wrapper around Element that implements Debug that will debugs the full element
///
/// This will hold a reference to the Document to get the name of the element and its attributes
pub struct ElementDebug<'element, 'doc> {
    pub(crate) element: &'element Element,
    pub(crate) doc: &'doc Document,
    // TODO: Possible depth limit?
}
impl<'element, 'doc> ElementDebug<'element, 'doc> {
    /// Create a new ElementDebug
    pub fn new(element: &'element Element, doc: &'doc Document) -> Self {
        Self { element, doc }
    }
}
impl Debug for ElementDebug<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { element, doc } = self;
        let name = element.name(doc);
        let attributes = element.attributes(doc);
        let children = element.children(doc);
        if children.is_empty() {
            return f
                .debug_struct("Element")
                .field("name", &name)
                .field("attributes", &attributes)
                .finish();
        }

        if children.len() == 1 && children[0].is_text() {
            // If it only has one child and that child is a text node, then we can just print the text
            let text = children[0].text_content(doc);
            return f
                .debug_struct("Element")
                .field("name", &name)
                .field("attributes", &attributes)
                .field("text", &text)
                .finish();
        }
        // Full debug of the children
        let children: Vec<_> = element
            .children(doc)
            .iter()
            .map(|child| child.debug(doc))
            .collect();
        f.debug_struct("Element")
            .field("name", &name)
            .field("attributes", &attributes)
            .field("children", &children)
            .finish()
    }
}
