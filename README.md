# edit-xml

edit-xml is a rust library to read, modify, and write XML documents. [Documentation](https://docs.rs/edit-xml/latest/edit-xml/)

It's aim is to be able to read **any** xml files, and modify only the parts you want to.

Features:

- Supports reading from most encodings, including UTF-16. (With the notable exception of UTF-32)
- You can have references to the parts of the tree, and still mutate the tree.
- Elements stores reference to its parent element, so traveling up the tree is fast.
- One of the fastest XML tree-like parser & writer. See [#Performance](https://github.com/bluegreenmagick/xml-doc#performance).
- Supports attribute value normalization, character/entity references.

Due to its architecture, you can't exchange nodes or elements between documents.
If your project modifies multiple xml documents at the same time, this library may not be a good fit.

## Example

```rust
use xml_doc::{Document, Element};

let XML = r#"<?xml version="1.0"?>
<package xmlns:dc="http://purl.org/dc/elements/1.1/">
    <metadata>
        <dc:title>xml-doc</dc:title>
        <dc:rights>MIT or Apache 2.0</dc:rights>
    </metadata>
</package>
"#;

let doc = Document::parse_str(XML).unwrap();
let package = doc.root_element().unwrap();
let metadata = package.find(&doc, "metadata").unwrap();
let title = metadata.find(&doc, "title").unwrap();
title.set_attribute("xml:lang", "en");

// Add an element to metadata: <dc:creator id="author">Yoonchae Lee</dc:creator>
let author = Element::build("dc:creator")
    .text_content("Yoonchae Lee")
    .attribute("id", "author")
    .push_to(&mut doc, metadata);

let new_xml = doc.write_str();
```

## Performance

To run benchmark: `cargo bench`.

### Tree-based Parser

```
                   tiny(5KB) medium(1.5MB) large(25MB) medium(UTF-16)
edit-xml v0.1.0:    81.02us     31.08ms      355.04ms      33.33ms
minidom v0.12.0:    94.93us     43.39ms      610.41ms
roxmltree v0.14.1:  52.73us     17.23ms      353.79ms
xmltree v0.10.3:  4305.7 us   1355.0 ms    22769.  ms
```
