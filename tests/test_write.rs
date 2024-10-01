use edit_xml::{Document, Element, Node};
mod test_utils;

#[test]
fn test_escape() -> anyhow::Result<()> {
    test_utils::setup_logger();
    let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<root attr="&amp;gt;&amp;lt;&amp;amp;&amp;quot;&amp;apos;attrval">
  <inner xmlns:ns="&amp;gt;&amp;lt;&amp;amp;&amp;quot;&amp;apos;nsval">&gt;&lt;&amp;&quot;&apos;text</inner>
</root>
<!--&lt;&amp;amp;--><![CDATA[<&amp;]]><!DOCTYPE  &lt;&amp;amp;>
<?<&amp;?>"#;

    let mut doc = Document::new();
    let container = doc.container();
    let root = Element::build("root")
        .attribute("attr", "><&\"'attrval")
        .push_to(&mut doc, container);
    Element::build("inner")
        .namespace_decl("ns", "><&\"'nsval")
        .add_text("><&\"'text")
        .push_to(&mut doc, root);
    doc.push_root_node(Node::Comment("<&amp;".to_string()))
        .unwrap();
    doc.push_root_node(Node::CData("<&amp;".to_string()))
        .unwrap();
    doc.push_root_node(Node::DocType("<&amp;".to_string()))
        .unwrap();
    doc.push_root_node(Node::PI("<&amp;".to_string())).unwrap();
    let xml = doc.write_str().unwrap();
    println!("{}", xml);
    assert_eq!(xml, expected);
    Ok(())
}
