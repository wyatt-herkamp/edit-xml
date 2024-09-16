use bench_utils::{bench, LARGE_XML, MEDIUM_UTF16, MEDIUM_XML, TINY_XML};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::File;
use std::path::Path;
use std::u32;
mod bench_utils;

//fn minidom_parse(path: &Path) {
// IT IS FUCKING CRASHING
//let mut reader =BufReader::new(File::open(path).unwrap());
//let doc = minidom::Element::from_reader(&mut reader).unwrap();
// black_box(doc);
//}
//bench!(TINY_XML, tiny_minidom, minidom_parse);
//bench!(MEDIUM_XML, medium_minidom, minidom_parse);
//bench!(LARGE_XML, large_minidom, minidom_parse);
//criterion_group!(minidom, tiny_minidom, medium_minidom, large_minidom);

fn edit_xml_parse(path: &Path) {
    let doc = edit_xml::Document::parse_file(path).unwrap();
    black_box(doc);
}
bench!(TINY_XML, tiny_edit_xml, edit_xml_parse);
bench!(MEDIUM_XML, medium_edit_xml, edit_xml_parse);
bench!(MEDIUM_UTF16, large_edit_xml, edit_xml_parse);
bench!(LARGE_XML, utf16_edit_xml, edit_xml_parse);

fn roxmltree_parse<'a>(path: &Path) {
    // roxmltree doesn't implement reading from reader.
    let xml = std::fs::read_to_string(path).unwrap();
    let doc = roxmltree::Document::parse_with_options(
        xml.as_ref(),
        roxmltree::ParsingOptions {
            allow_dtd: true,
            nodes_limit: u32::MAX,
        },
    )
    .unwrap();
    black_box(doc);
}
bench!(TINY_XML, tiny_roxmltree, roxmltree_parse);
bench!(MEDIUM_XML, medium_roxmltree, roxmltree_parse);
bench!(LARGE_XML, large_roxmltree, roxmltree_parse);
criterion_group!(roxmltree, tiny_roxmltree, medium_roxmltree, large_roxmltree);

fn xmltree_parse(path: &Path) {
    let file = File::open(path).unwrap();
    let doc = xmltree::Element::parse(file).unwrap();
    black_box(doc);
}
bench!(TINY_XML, tiny_xmltree, xmltree_parse);
bench!(MEDIUM_XML, medium_xmltree, xmltree_parse);
bench!(LARGE_XML, large_xmltree, xmltree_parse);
criterion_group!(xmltree, tiny_xmltree, medium_xmltree, large_xmltree);

criterion_group! {
    name = tiny;
    config = Criterion::default().sample_size(200);
    targets = tiny_edit_xml,  tiny_roxmltree, tiny_xmltree
}

criterion_group!(medium, medium_edit_xml, medium_roxmltree, medium_xmltree,);

criterion_group! {
    name = large;
    config = Criterion::default().sample_size(50);
    targets = large_edit_xml,  large_roxmltree
}

criterion_group!(utf_16, utf16_edit_xml);

criterion_group!(
    edit_xml,
    tiny_edit_xml,
    medium_edit_xml,
    large_edit_xml,
    utf16_edit_xml
);

criterion_main!(tiny, medium, large, utf_16);
