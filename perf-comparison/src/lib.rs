#![allow(dead_code)]

pub static TINY_XML: &str = "tiny.hidden.xml";
pub static MEDIUM_XML: &str = "medium.hidden.xml";
pub static MEDIUM_UTF16: &str = "medium_utf16.hidden.xml";
pub static LARGE_XML: &str = "large.hidden.xml";
#[macro_export]
macro_rules! bench {
    ($filename:literal, $name:ident, $func:path) => {
        pub fn $name(c: &mut criterion::Criterion) {
            let path = std::path::Path::new($filename);
            c.bench_function(stringify!($name), |b| b.iter(|| $func(&path)));
        }
    };
    ($file:ident, $name:ident, $func:path) => {
        pub fn $name(c: &mut criterion::Criterion) {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("bench_files")
                .join($file);
            c.bench_function(stringify!($name), |b| b.iter(|| $func(&path)));
        }
    };
}
pub fn get_bench_file_path(file: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bench_files")
        .join(file)
}
pub fn open_bench_file(file: &str) -> std::fs::File {
    std::fs::File::open(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("bench_files")
            .join(file),
    )
    .unwrap()
}
#[cfg(test)]
mod edit_xml_tests {
    use std::hint::black_box;

    use crate::get_bench_file_path;

    #[test]
    fn tiny_xml() {
        let file = get_bench_file_path(crate::TINY_XML);
        let string = std::fs::read_to_string(file).unwrap();

        for _ in 0..1000 {
            let document = edit_xml::Document::parse_str(&string).unwrap();
            black_box(document);
        }
    }
    #[test]
    fn medium_xml() {
        let file = get_bench_file_path(crate::MEDIUM_XML);
        let string = std::fs::read_to_string(file).unwrap();

        for _ in 0..1000 {
            let document = edit_xml::Document::parse_str(&string).unwrap();
            black_box(document);
        }
    }
}
