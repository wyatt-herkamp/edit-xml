use bench_utils::{bench, LARGE_XML, MEDIUM_XML, TINY_XML};
use criterion::{criterion_group, criterion_main, Criterion};
use std::fs::File;
use std::path::Path;
mod bench_utils;

mod xml_rs_bench {
    use super::*;
    use std::io::BufReader;
    use xml_rs::reader::{EventReader, XmlEvent};

    pub fn xml_rs_parser(path: &Path) -> usize {
        let mut counter = 0;
        let file = File::open(path).unwrap();
        let file = BufReader::new(file);

        let parser = EventReader::new(file);
        for e in parser {
            match e {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    counter += name.local_name.len();
                    for attr in attributes {
                        counter += attr.value.len();
                    }
                }
                _ => (),
            }
        }
        counter
    }
}
use xml_rs_bench::xml_rs_parser;

bench!(TINY_XML, tiny_xml_rs, xml_rs_parser);
bench!(MEDIUM_XML, medium_xml_rs, xml_rs_parser);
bench!(LARGE_XML, large_xml_rs, xml_rs_parser);
criterion_group!(xml_rs, tiny_xml_rs, medium_xml_rs, large_xml_rs);

/// Parser Benchmarking
/// Calculates sum of tag local name byte length and attribute values byte length
///
/// Because quick_xml just stores the whole tag content as bytes without processing it
/// the below test should take that into consideration as it makes it calculate tag name and attributes.
//////////////////////////////////////////

fn quick_xml_parser(path: &Path) -> usize {
    let mut count = 0;
    let mut reader = quick_xml::Reader::from_file(path).unwrap();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(tag)) | Ok(quick_xml::events::Event::Empty(tag)) => {
                count += 1;
                // WTF
                for attr in tag.attributes() {
                    let attr = attr.unwrap();
                    count += attr.value.len();
                }
            }
            Ok(quick_xml::events::Event::Eof) => {
                break;
            }
            _ => (),
        }
    }
    count
}

bench!(TINY_XML, tiny_quick_xml, quick_xml_parser);
bench!(MEDIUM_XML, medium_quick_xml, quick_xml_parser);
bench!(LARGE_XML, large_quick_xml, quick_xml_parser);
criterion_group!(quick_xml, tiny_quick_xml, medium_quick_xml, large_quick_xml);

mod xml5ever_bench {
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;

    use super::*;
    use markup5ever::buffer_queue::BufferQueue;
    use xml5ever::tendril::*;
    use xml5ever::tokenizer::{TagToken, Token, TokenSink, XmlTokenizer};
    #[derive(Clone)]
    struct TokenCounter {
        counter: Arc<AtomicUsize>,
    }

    impl TokenSink for TokenCounter {
        fn process_token(&self, token: Token) {
            // THE NEWEST VERSION ISNT MUTABLE.  So Using an Arc to make things less annoying
            match token {
                TagToken(tag) => {
                    let mut add = tag.name.local.as_ref().len();
                    for attr in tag.attrs {
                        add += attr.value.len()
                    }
                    self.counter
                        .fetch_add(add, std::sync::atomic::Ordering::Relaxed);
                }
                _ => (),
            }
        }
    }

    pub fn xml5ever_parser(path: &Path) -> usize {
        let sink = TokenCounter {
            counter: Arc::new(AtomicUsize::default()),
        };

        let mut file = File::open(&path).ok().expect("can't open file");
        let mut input = ByteTendril::new();
        file.read_to_tendril(&mut input)
            .ok()
            .expect("can't read file");
        let mut input_buffer = BufferQueue::default();
        input_buffer.push_back(input.try_reinterpret().unwrap());
        let tok = XmlTokenizer::new(sink.clone(), Default::default());
        tok.feed(&mut input_buffer);
        tok.end();

        return sink.counter.load(std::sync::atomic::Ordering::Relaxed);
    }
}
use xml5ever_bench::xml5ever_parser;

bench!(TINY_XML, tiny_xml5ever, xml5ever_parser);
bench!(MEDIUM_XML, medium_xml5ever, xml5ever_parser);
bench!(LARGE_XML, large_xml5ever, xml5ever_parser);
criterion_group!(xml5ever, tiny_xml5ever, medium_xml5ever, large_xml5ever);

criterion_main!(xml_rs, quick_xml, xml5ever);
