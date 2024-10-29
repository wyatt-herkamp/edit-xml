use crate::{Document, Element, Node};

impl Element {
    /// Sorts children of this element using `compare` function.
    ///
    /// # Note
    ///
    /// All children are cloned in this method.
    /// This is due to the fact that we have to pass the document to the compare function while holding the children.
    ///
    /// This method is not stable and may change in the future.
    ///
    /// # Example
    /// ```
    /// use edit_xml::{Document, Element};
    /// let xml = r#"<?xml version="1.0"?>
    ///         <root>
    ///             <c>3</c>
    ///             <b>2</b>
    ///             <a>1</a>
    ///         </root>
    /// "#;
    /// let mut doc = Document::parse_str(xml).unwrap();
    /// let root = doc.root_element().unwrap();
    /// assert_eq!(root.children(&doc)[0].text_content(&doc), "3");
    /// assert_eq!(root.children(&doc)[1].text_content(&doc), "2");
    /// assert_eq!(root.children(&doc)[2].text_content(&doc), "1");
    /// root.sort_children_cloned(&mut doc, |doc, a, b| {
    ///    let a = a.text_content(doc).parse::<i32>().unwrap();
    ///    let b = b.text_content(doc).parse::<i32>().unwrap();
    ///    a.cmp(&b)
    /// });
    /// let children = root.children(&doc);
    /// assert_eq!(children[0].text_content(&doc), "1");
    /// assert_eq!(children[1].text_content(&doc), "2");
    /// assert_eq!(children[2].text_content(&doc), "3");
    /// let write = doc.write_str().unwrap();
    ///
    /// let doc = Document::parse_str(&write).unwrap();
    /// let root = doc.root_element().unwrap();
    /// let children = root.children(&doc);
    /// assert_eq!(children[0].text_content(&doc), "1");
    /// assert_eq!(children[1].text_content(&doc), "2");
    /// assert_eq!(children[2].text_content(&doc), "3");
    /// ```
    pub fn sort_children_cloned<'a, F>(&'a self, doc: &'a mut Document, compare: F)
    where
        F: Fn(&Document, &Node, &Node) -> std::cmp::Ordering,
    {
        let mut cloned = self.data(doc).children.clone();
        cloned.sort_by(|a, b| compare(doc, a, b));
        self.mut_data(doc).children = cloned;
    }
}
