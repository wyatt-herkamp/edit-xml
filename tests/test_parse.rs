use edit_xml::{Document, EditXMLError, Node, ReadOptions};
use quick_xml::errors::IllFormedError;
use tracing::{debug, info};
fn setup_logger() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let stdout_log = tracing_subscriber::fmt::layer().pretty();
        tracing_subscriber::registry().with(stdout_log).init();
    });
    println!("Logger initialized");
    info!("Logger initialized");
    debug!("Logger initialized");
}

#[test]
fn test_normalize_attr() {
    setup_logger();
    // See comment on xml_doc::parser::DocumentParser::normalize_attr_value
    let xml = "<?xml version=\"1.0\"?>
<root attr=\" \r\t

 ab&#xD;   c
  \" />";
    let doc = Document::parse_str(xml).unwrap();
    let root = doc.root_element().unwrap();
    let val = root.attribute(&doc, "attr").unwrap();

    assert_eq!(val, "ab\r c");
}

#[test]
fn test_closing_tag_mismatch_err() {
    setup_logger();

    // no closing tag
    let xml = "<img>";
    let mut opts = ReadOptions::default();
    opts.require_decl = false;
    let doc = Document::parse_str_with_opts(xml, opts.clone());
    assert!(doc.is_err());

    // closing tag mismatch
    let xml = "<a><img>Te</a>xt</img>";
    let doc = Document::parse_str_with_opts(xml, opts.clone());
    assert!(doc.is_err());

    // no opening tag
    let xml = "</abc>";
    let doc = Document::parse_str_with_opts(xml, opts.clone());
    assert!(doc.is_err());
}

#[test]
fn test_unescape() {
    setup_logger();

    let xml = r#"<abc attr="&quot;val&quot;">&lt;Text&amp;&apos;&gt;</abc>
    <![CDATA[<&amp;>]]>
    <!-- <&amp; cmt -->
    <!DOCTYPE &amp;>
    <?<&amp;?>"#;
    let mut opts = ReadOptions::default();
    opts.require_decl = false;
    let doc = Document::parse_str_with_opts(xml, opts).unwrap();

    let abc = doc.root_element().unwrap();
    assert_eq!(abc.attribute(&doc, "attr"), Some("\"val\""));
    let text = &abc.children(&doc)[0];
    assert!(matches!(text, Node::Text(_)));
    assert_eq!(text.text_content(&doc), "<Text&'>");

    let cdata = &doc.root_nodes()[1];
    assert!(matches!(cdata, Node::CData(_)));
    assert_eq!(cdata.text_content(&doc), "<&amp;>");

    let comment = &doc.root_nodes()[2];
    if let Node::Comment(cmt) = comment {
        assert_eq!(cmt, " <&amp; cmt ");
    } else {
        assert!(false);
    }

    let doctype = &doc.root_nodes()[3];
    if let Node::DocType(doc) = doctype {
        assert_eq!(doc, "&");
    } else {
        assert!(false);
    }

    let pi = &doc.root_nodes()[4];
    assert!(matches!(pi, Node::PI(_)));
    assert_eq!(pi.text_content(&doc), "<&amp;");
}
