use crate::element::{Element, ElementData};
use crate::error::{EditXMLError, Result};
use crate::parser::{DocumentParser, ReadOptions};
use crate::types::StandaloneValue;
use crate::ElementBuilder;
use quick_xml::events::{BytesCData, BytesDecl, BytesEnd, BytesPI, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;
#[cfg(feature = "document-breakdown")]
mod breakdown;
#[cfg(feature = "document-breakdown")]
pub use breakdown::*;
mod node;
pub use node::*;

/// Represents a XML document or a document fragment.
///
/// To build a document from scratch, use [`Document::new`].
///
/// To read and modify an existing document, use [parse_*](`Document#parsing`) methods.
///
/// To write the document, use [write_*](`Document#writing`) methods.
///
/// # Examples
/// ```
/// use edit_xml::Document;
///
/// let mut doc = Document::parse_str(r#"<?xml version="1.0" encoding="UTF-8"?>
/// <package>
///     <metadata>
///         <author>Lewis Carol</author>
///     </metadata>
/// </package>
/// "#).unwrap();
/// let author_elem = doc
///   .root_element()
///   .unwrap()
///   .find(&doc, "metadata")
///   .unwrap()
///   .find(&doc, "author")
///   .unwrap();
/// author_elem.set_text_content(&mut doc, "Lewis Carroll");
/// let xml = doc.write_str();
/// ```
///

#[derive(Debug)]
pub struct Document {
    pub(crate) counter: usize, // == self.store.len()
    pub(crate) store: Vec<ElementData>,
    container: Element,

    pub(crate) version: String,
    pub(crate) standalone: Option<StandaloneValue>,
}
impl Default for Document {
    fn default() -> Self {
        let (container, container_data) = Element::container();
        Document {
            counter: 1, // because container is id 0
            store: vec![container_data],
            container,
            version: String::from("1.0"),
            standalone: None,
        }
    }
}
impl Document {
    /// Create a blank new xml document.
    pub fn new() -> Document {
        Document::default()
    }
    pub fn new_with_store_size(size: usize) -> Document {
        let (container, container_data) = Element::container();
        let mut store = Vec::with_capacity(size + 1);
        store.push(container_data);
        Document {
            counter: 1, // because container is id 0
            store,
            container,
            version: String::from("1.0"),
            standalone: None,
        }
    }
    /// Get the number of elements in the document.
    pub fn number_of_elements(&self) -> usize {
        self.store.len()
    }
    /// Create a new xml document with a root element.
    ///
    /// # Examples
    /// ```
    /// use edit_xml::Document;
    /// let mut doc = Document::new_with_root("root", |root| {
    ///    root.attribute("id", "main")
    ///     .attribute("class", "main")
    ///     .create_element("name", |elem| {
    ///         elem.add_text("Cool Name")
    ///     })
    /// });
    /// let root = doc.root_element().unwrap();
    /// let name = root.find(&doc, "name").unwrap();
    /// assert_eq!(name.text_content(&doc), "Cool Name");
    /// ```
    pub fn new_with_root<N, F>(root_name: N, f: F) -> Document
    where
        N: Into<String>,
        F: FnOnce(ElementBuilder) -> ElementBuilder,
    {
        let mut doc = Document::new();
        let root = f(ElementBuilder::new(root_name)).finish(&mut doc);
        doc.push_root_node(root).unwrap();
        doc
    }

    /// Get 'container' element of Document.
    ///
    /// The document uses an invisible 'container' element
    /// which it uses to manage its root nodes.
    ///
    /// Its parent is None, and trying to change its parent will
    /// result in an error.
    ///
    /// For the container element, only its `children` is relevant.
    /// Other attributes are not used.
    pub fn container(&self) -> Element {
        self.container
    }

    /// Returns `true` if document doesn't have any nodes.
    /// Returns `false` if you added a node or parsed an xml.
    ///
    /// You can only call `parse_*()` if document is empty.
    pub fn is_empty(&self) -> bool {
        self.store.len() == 1
    }

    /// Get root nodes of document.
    pub fn root_nodes(&self) -> &Vec<Node> {
        self.container.children(self)
    }

    /// Get first root node that is an element.
    pub fn root_element(&self) -> Option<Element> {
        self.container.child_elements(self).first().copied()
    }

    /// Push a node to end of root nodes.
    /// If doc has no [`Element`], pushing a [`Node::Element`] is
    /// equivalent to setting it as root element.
    pub fn push_root_node(&mut self, node: impl Into<Node>) -> Result<()> {
        let node = node.into();
        let elem = self.container;
        elem.push_child(self, node)
    }
    #[inline(always)]
    pub(crate) fn push_to_store(&mut self, data: ElementData) -> Element {
        let elem = Element { id: self.counter };
        self.counter += 1;
        self.store.push(data);
        elem
    }
}

/// &nbsp;
/// # Parsing
///
/// Below are methods for parsing xml.
/// Parsing from string, file, and reader is supported.
///
/// Call `parse_*_with_opts` with custom [`ReadOptions`] to change parser behaviour.
/// Otherwise, [`ReadOptions::default()`] is used.
///
impl Document {
    pub fn parse_str(str: &str) -> Result<Document> {
        DocumentParser::parse_reader(str.as_bytes(), ReadOptions::default())
    }
    pub fn parse_str_with_opts(str: &str, opts: ReadOptions) -> Result<Document> {
        DocumentParser::parse_reader(str.as_bytes(), opts)
    }

    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Document> {
        let file = File::open(path)?;
        DocumentParser::parse_reader(file, ReadOptions::default())
    }
    pub fn parse_file_with_opts<P: AsRef<Path>>(path: P, opts: ReadOptions) -> Result<Document> {
        let file = File::open(path)?;
        DocumentParser::parse_reader(file, opts)
    }

    pub fn parse_reader<R: Read>(reader: R) -> Result<Document> {
        DocumentParser::parse_reader(reader, ReadOptions::default())
    }
    pub fn parse_reader_with_opts<R: Read>(reader: R, opts: ReadOptions) -> Result<Document> {
        DocumentParser::parse_reader(reader, opts)
    }
}

/// Options when writing XML.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WriteOptions {
    /// Byte character to indent with. (default: `b' '`)
    pub indent_char: u8,
    /// How many indent_char should be used for indent. (default: 2)
    pub indent_size: usize,
    /// XML declaration should be written at the top. (default: `true`)
    pub write_decl: bool,
}
impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            indent_char: b' ',
            indent_size: 2,
            write_decl: true,
        }
    }
}

/// &nbsp;
/// # Writing
///
/// Below are methods for writing xml.
/// The XML will be written in UTF-8.
impl Document {
    pub fn write_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.write_file_with_opts(path, WriteOptions::default())
    }
    pub fn write_file_with_opts<P: AsRef<Path>>(&self, path: P, opts: WriteOptions) -> Result<()> {
        let mut file = File::open(path)?;
        self.write_with_opts(&mut file, opts)
    }

    pub fn write_str(&self) -> Result<String> {
        self.write_str_with_opts(WriteOptions::default())
    }
    pub fn write_str_with_opts(&self, opts: WriteOptions) -> Result<String> {
        let mut buf: Vec<u8> = Vec::with_capacity(200);
        self.write_with_opts(&mut buf, opts)?;
        Ok(String::from_utf8(buf)?)
    }

    pub fn write(&self, writer: &mut impl Write) -> Result<()> {
        self.write_with_opts(writer, WriteOptions::default())
    }
    pub fn write_with_opts(&self, writer: &mut impl Write, opts: WriteOptions) -> Result<()> {
        let container = self.container();
        let mut writer = Writer::new_with_indent(writer, opts.indent_char, opts.indent_size);
        if opts.write_decl {
            self.write_decl(&mut writer)?;
        }
        self.write_nodes(&mut writer, container.children(self))?;
        writer.write_event(Event::Eof)?;
        Ok(())
    }

    fn write_decl(&self, writer: &mut Writer<impl Write>) -> Result<()> {
        let standalone = self.standalone.map(|v| v.as_str());
        writer.write_event(Event::Decl(BytesDecl::new(
            &self.version,
            Some("UTF-8"),
            standalone,
        )))?;
        Ok(())
    }

    fn write_nodes(&self, writer: &mut Writer<impl Write>, nodes: &[Node]) -> Result<()> {
        for node in nodes {
            match node {
                Node::Element(eid) => self.write_element(writer, *eid)?,
                Node::Text(text) => writer.write_event(Event::Text(BytesText::new(text)))?,
                Node::DocType(text) => writer.write_event(Event::DocType(
                    BytesText::new(&format!(" {}", text)), // add a whitespace before text
                ))?,
                // Comment, CData, and PI content is not escaped.
                Node::Comment(text) => {
                    // Unescaped Text??
                    writer.write_event(Event::Comment(BytesText::new(text)))?
                }
                Node::CData(text) => {
                    // Escaped Text ??
                    writer.write_event(Event::CData(BytesCData::new(text)))?
                }
                Node::PI(text) => writer.write_event(Event::PI(BytesPI::new(text)))?,
            };
        }
        Ok(())
    }

    fn write_element(&self, writer: &mut Writer<impl Write>, element: Element) -> Result<()> {
        let name_bytes = element.full_name(self);
        let mut start = BytesStart::new(name_bytes);
        for (key, val) in element.attributes(self) {
            let val = quick_xml::escape::escape(val.as_str());
            start.push_attribute((key.as_str(), val.as_ref()));
        }
        for (prefix, val) in element.namespace_decls(self) {
            let attr_name = if prefix.is_empty() {
                "xmlns".to_string()
            } else {
                format!("xmlns:{}", prefix)
            };

            let val = quick_xml::escape::escape(val.as_str());
            start.push_attribute((attr_name.as_str(), val.as_ref()));
        }
        if element.has_children(self) {
            writer.write_event(Event::Start(start))?;
            self.write_nodes(writer, element.children(self))?;
            writer.write_event(Event::End(BytesEnd::new(name_bytes)))?;
        } else {
            writer.write_event(Event::Empty(start))?;
        }
        Ok(())
    }
}

impl FromStr for Document {
    type Err = EditXMLError;

    fn from_str(s: &str) -> Result<Document> {
        Document::parse_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_element() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <basic>
            Text
            <c />
        </basic>
        "#;
        let mut doc = Document::from_str(xml).unwrap();
        let basic = doc.container().children(&doc)[0].as_element().unwrap();
        let p = Element::new(&mut doc, "p");
        basic.push_child(&mut doc, Node::Element(p)).unwrap();
        assert_eq!(p.parent(&doc).unwrap(), basic);
        assert_eq!(
            p,
            basic.children(&doc).last().unwrap().as_element().unwrap()
        )
    }
}
