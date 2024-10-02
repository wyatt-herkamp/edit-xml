use crate::{utils::HashMap, Document, Node};

use super::Element;
#[derive(Debug, Clone, PartialEq, Eq)]
enum NewNodes {
    Element(ElementBuilder),
    Text(String),
    Comment(String),
    CData(String),
    PI(String),
}
impl NewNodes {
    /// Converts te NewNode into a Node and pushes it to the parent.
    ///
    /// # Panics
    /// If the parent is not an element.
    fn push_to(self, doc: &mut Document, parent: Element) {
        let result = match self {
            NewNodes::Element(elem) => {
                let elem = elem.finish(doc);
                parent.push_child(doc, Node::Element(elem))
            }
            NewNodes::Text(text) => parent.push_child(doc, Node::Text(text)),
            NewNodes::Comment(text) => parent.push_child(doc, Node::Comment(text)),
            NewNodes::CData(text) => parent.push_child(doc, Node::CData(text)),
            NewNodes::PI(text) => parent.push_child(doc, Node::PI(text)),
        };

        if let Err(e) = result {
            panic!("Illegal Parameter put in ElementBuilder: {:?}", e);
        }
    }
}
/// An easy way to build a new element
/// by chaining methods to add properties.
///
/// Call [`Element::build()`] to start building.
/// To finish building, either call `.finish()` or `.push_to(parent)`
/// which returns [`Element`].
///
/// # Examples
///
/// ```
/// use edit_xml::{Document, Element, Node};
///
/// let mut doc = Document::new();
///
/// let root = Element::build("root")
///     .attribute("id", "main")
///     .attribute("class", "main")
///     .finish(&mut doc);
/// doc.push_root_node(root.as_node());
///
/// let name = Element::build("name")
///     .add_text("No Name")
///     .push_to(&mut doc, root);
///
/// /* Equivalent xml:
///   <root id="main" class="main">
///     <name>No Name</name>
///   </root>
/// */
/// ```
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementBuilder {
    full_name: String,
    attributes: HashMap<String, String>,
    namespace_decls: HashMap<String, String>,
    content: Vec<NewNodes>,
}
macro_rules! push_node {
    (
        $(
            $(#[$docs:meta])*
            $fn_name:ident => $node:ident
        ),*
    ) => {
        $(
            $(#[$docs])*
            pub fn $fn_name<S: Into<String>>(self, text: S) -> Self {
                self.push_node(NewNodes::$node(text.into()))
            }
        )*
    };
}
impl ElementBuilder {
    /// Creates a new ElementBuilder with the full name of the element.
    pub fn new(full_name: impl Into<String>) -> ElementBuilder {
        ElementBuilder::new_with_capacities(full_name, 0, 0, 0)
    }
    /// Creates a new ElementBuilder with the full name of the element and the capacities of the attributes, namespace declarations, and content.
    pub fn new_with_capacities(
        full_name: impl Into<String>,
        attribute_capacity: usize,
        namespace_capacity: usize,
        content_capacity: usize,
    ) -> ElementBuilder {
        ElementBuilder {
            full_name: full_name.into(),
            attributes: HashMap::with_capacity(attribute_capacity),
            namespace_decls: HashMap::with_capacity(namespace_capacity),
            content: Vec::with_capacity(content_capacity),
        }
    }
    /// Removes previous prefix if it exists, and attach new prefix.
    pub fn prefix(mut self, prefix: &str) -> Self {
        let (_, name) = Element::separate_prefix_name(&self.full_name);
        if prefix.is_empty() {
            self.full_name = name.to_string();
        } else {
            self.full_name = format!("{}{}", prefix, name);
        }
        self
    }
    /// Add an attribute to the element.
    pub fn attribute<S, T>(mut self, name: S, value: T) -> Self
    where
        S: Into<String>,
        T: Into<String>,
    {
        self.attributes.insert(name.into(), value.into());
        self
    }
    /// Add a namespace declaration to the element.
    pub fn namespace_decl<S, T>(mut self, prefix: S, namespace: T) -> Self
    where
        S: Into<String>,
        T: Into<String>,
    {
        self.namespace_decls.insert(prefix.into(), namespace.into());
        self
    }

    fn push_node(mut self, node: NewNodes) -> Self {
        self.content.push(node);
        self
    }

    push_node![
        /// Add text content to the element.
        ///
        /// ```
        /// use edit_xml::{Document, Element};
        /// let mut doc = Document::new();
        /// let root = Element::build("root")
        ///    .add_text("Hello")
        ///    .finish(&mut doc);
        /// let content = root.children(&doc);
        /// assert_eq!(content.len(), 1);
        /// assert!(content[0].is_text());
        /// ```
        add_text => Text,
        /// Add a comment node to the element.
        ///
        /// ```
        /// use edit_xml::{Document, Element};
        /// let mut doc = Document::new();
        /// let root = Element::build("root")
        ///   .add_comment("This is a comment")
        ///  .finish(&mut doc);
        /// let content = root.children(&doc);
        /// assert_eq!(content.len(), 1);
        /// assert!(content[0].is_comment());
        /// ```
        add_comment => Comment,
        /// Add a CDATA node to the element.
        /// ```
        /// use edit_xml::{Document, Element};
        /// let mut doc = Document::new();
        /// let root = Element::build("root")
        ///   .add_cdata("This is a CDATA")
        /// .finish(&mut doc);
        /// let content = root.children(&doc);
        /// assert_eq!(content.len(), 1);
        /// assert!(content[0].is_cdata());
        /// ```
        add_cdata => CData,
        /// Add a Processing Instruction node to the element.
        ///
        /// ```
        /// use edit_xml::{Document, Element};
        /// let mut doc = Document::new();
        /// let root = Element::build("root")
        ///  .add_pi("target")
        /// .finish(&mut doc);
        /// let content = root.children(&doc);
        /// assert_eq!(content.len(), 1);
        /// assert!(content[0].is_pi());
        /// ```
        add_pi => PI
    ];
    /// Add an element to the element.
    pub fn add_element(mut self, elem: ElementBuilder) -> Self {
        self.content.push(NewNodes::Element(elem));
        self
    }
    /// Adds an element to the element.
    ///
    /// ```
    /// use edit_xml::{Document, Element};
    /// let mut doc = Document::new();
    ///
    /// let root = Element::build("root")
    ///    .create_element("child", |builder| {
    ///       builder
    ///          .add_text("Hello")
    /// }).finish(&mut doc);
    ///
    /// let content = root.children(&doc);
    /// assert_eq!(content.len(), 1);
    /// assert_eq!(content[0].text_content(&doc), "Hello");
    /// ```
    pub fn create_element<F>(self, name: impl Into<String>, f: F) -> Self
    where
        F: FnOnce(ElementBuilder) -> ElementBuilder,
    {
        let builder = f(ElementBuilder::new(name));
        self.add_element(builder)
    }
    /// Finish building the element and return it.
    /// The result must be pushed to a parent element or the root node.
    ///
    /// ```
    /// use edit_xml::{Document, Element, ElementBuilder};
    /// let mut doc = Document::new();
    /// let root = ElementBuilder::new("root")
    ///   .add_text("Hello")
    ///   .finish(&mut doc);
    /// doc.push_root_node(root.as_node());
    /// ```
    #[must_use]
    pub fn finish(self, doc: &mut Document) -> Element {
        let Self {
            full_name,
            attributes,
            namespace_decls,
            content,
        } = self;
        let elem = Element::with_data(doc, full_name, attributes, namespace_decls);

        for node in content {
            node.push_to(doc, elem);
        }
        elem
    }

    /// Push this element to the parent's children.
    ///
    /// # Panics
    ///
    /// If the parent is not an element.
    pub fn push_to(self, doc: &mut Document, parent: Element) -> Element {
        let elem = self.finish(doc);
        parent
            .push_child_element(doc, elem)
            .expect("Illegal Parameter put in ElementBuilder");
        elem
    }
    /// Push this element to the root node.
    /// ```
    /// use edit_xml::{Document, Element, ElementBuilder};
    /// let mut doc = Document::new();
    /// let root = ElementBuilder::new("root")
    ///   .add_text("Hello")
    ///   .push_to_root_node(&mut doc);
    /// assert_eq!(doc.root_element().unwrap(), root);
    pub fn push_to_root_node(self, doc: &mut Document) -> Element {
        let elem = self.finish(doc);
        doc.push_root_node(Node::Element(elem))
            .expect("Illegal Parameter put in ElementBuilder");
        elem
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::Document;
    fn start() -> Document {
        crate::utils::tests::setup_logger();

        Document::new()
    }
    #[test]
    fn test_element_builder() -> anyhow::Result<()> {
        let mut doc = start();

        let element = ElementBuilder::new("root")
            .attribute("id", "main")
            .attribute("class", "main")
            .create_element("tests", |new| {
                new.create_element("child", |new| new.add_text("Hello"))
            })
            .add_comment("This is a comment")
            .push_to_root_node(&mut doc);

        let new_element = ElementBuilder::new("hello")
            .add_text("world")
            .finish(&mut doc);
        element.push_child(&mut doc, new_element)?;
        let to_string = doc.write_str()?;

        println!("{}", to_string);

        Ok(())
    }
}
