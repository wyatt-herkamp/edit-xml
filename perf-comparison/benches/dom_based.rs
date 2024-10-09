#![allow(dead_code)]
use criterion::{criterion_group, criterion_main, Criterion};

mod minidom_bench {
    use minidom::*;
    use std::{fs::File, io::BufReader, path::Path};

    use criterion::black_box;
    use xml_lib_perf_comparison::{bench, LARGE_XML, MEDIUM_XML, TINY_XML};

    fn minidom_parse(path: &Path) {
        // IT IS FUCKING CRASHING
        let mut reader = BufReader::new(File::open(path).unwrap());
        let doc = Element::from_reader(&mut reader).unwrap();
        black_box(doc);
    }
    bench!(TINY_XML, tiny_minidom, minidom_parse);
    bench!(MEDIUM_XML, medium_minidom, minidom_parse);
    bench!(LARGE_XML, large_minidom, minidom_parse);
}

mod edit_xml_bench {
    use criterion::black_box;
    use edit_xml::*;
    use std::path::Path;
    use xml_lib_perf_comparison::{bench, LARGE_XML, MEDIUM_UTF16, MEDIUM_XML, TINY_XML};

    fn edit_xml_parse(path: &Path) {
        let doc = Document::parse_file(path).unwrap();
        black_box(doc);
    }
    bench!(TINY_XML, tiny_edit_xml, edit_xml_parse);
    bench!(MEDIUM_XML, medium_edit_xml, edit_xml_parse);
    bench!(MEDIUM_UTF16, large_edit_xml, edit_xml_parse);
    bench!(LARGE_XML, utf16_edit_xml, edit_xml_parse);
}

mod roxmltree_bench {
    use criterion::black_box;
    use std::path::Path;
    use xml_lib_perf_comparison::{bench, LARGE_XML, MEDIUM_XML, TINY_XML};

    fn roxmltree_parse(path: &Path) {
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
}
mod xmltree_bench {
    use criterion::black_box;
    use std::{fs::File, path::Path};
    use xml_lib_perf_comparison::{bench, LARGE_XML, MEDIUM_XML, TINY_XML};

    fn xmltree_parse(path: &Path) {
        let file = File::open(path).unwrap();
        let doc = xmltree::Element::parse(file).unwrap();
        black_box(doc);
    }
    bench!(TINY_XML, tiny_xmltree, xmltree_parse);
    bench!(MEDIUM_XML, medium_xmltree, xmltree_parse);
    bench!(LARGE_XML, large_xmltree, xmltree_parse);
}

criterion_group! {
    name = tiny;
    config = Criterion::default().sample_size(200);
    targets = edit_xml_bench::tiny_edit_xml, roxmltree_bench::tiny_roxmltree, xmltree_bench::tiny_xmltree
}

criterion_group! {
    name = medium;
    config = Criterion::default().sample_size(100);
    targets = edit_xml_bench::medium_edit_xml, roxmltree_bench::medium_roxmltree, xmltree_bench::medium_xmltree
}

criterion_group! {
    name = large;
    config = Criterion::default().sample_size(10);
    targets = edit_xml_bench::large_edit_xml, roxmltree_bench::large_roxmltree, xmltree_bench::large_xmltree
}
criterion_group!(utf_16, edit_xml_bench::utf16_edit_xml);

criterion_main!(tiny, medium, large, utf_16);
