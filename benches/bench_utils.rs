#![allow(dead_code)]
pub static TINY_XML: &str = "tiny.hidden.xml";
pub static MEDIUM_XML: &str = "medium.hidden.xml";
pub static MEDIUM_UTF16: &str = "medium_utf16.hidden.xml";
pub static LARGE_XML: &str = "large.hidden.xml";

macro_rules! bench {
    ($filename:literal, $name:ident, $func:path) => {
        fn $name(c: &mut Criterion) {
            let path = Path::new($filename);
            c.bench_function(stringify!($name), |b| b.iter(|| $func(&path)));
        }
    };
    ($file:ident, $name:ident, $func:path) => {
        fn $name(c: &mut Criterion) {
            let path = std::path::Path::new("benches")
                .join("bench_files")
                .join($file);
            c.bench_function(stringify!($name), |b| b.iter(|| $func(&path)));
        }
    };
}
pub(crate) use bench;
