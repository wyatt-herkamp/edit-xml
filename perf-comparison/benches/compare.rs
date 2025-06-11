#![allow(dead_code)]
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use edit_xml::ReadOptions;
use xml_lib_perf_comparison::{get_bench_file_path, LARGE_XML, MEDIUM_XML, TINY_XML};

fn compare_tiny(c: &mut Criterion) {
    let path = get_bench_file_path(TINY_XML);
    let xml = std::fs::read_to_string(path).unwrap();
    let mut group = c.benchmark_group("tiny_compare");

    group.bench_function(BenchmarkId::new("edit_xml", TINY_XML), |b| {
        b.iter(|| {
            let doc = edit_xml::Document::parse_str_with_opts(
                &xml,
                ReadOptions {
                    ..Default::default()
                },
            )
            .unwrap();
            black_box(doc);
        })
    });

    group.bench_function(BenchmarkId::new("xml_tree", TINY_XML), |b| {
        b.iter(|| {
            let doc = xmltree::Element::parse(xml.as_bytes()).unwrap();
            black_box(doc);
        })
    });
    group.bench_function(BenchmarkId::new("roxmltree", TINY_XML), |b| {
        b.iter(|| {
            let doc = roxmltree::Document::parse_with_options(
                xml.as_ref(),
                roxmltree::ParsingOptions {
                    allow_dtd: true,
                    nodes_limit: u32::MAX,
                },
            )
            .unwrap();
            black_box(doc);
        })
    });
    group.finish();
}

fn compare_medium(c: &mut Criterion) {
    let path = get_bench_file_path(MEDIUM_XML);
    let xml = std::fs::read_to_string(path).unwrap();
    let mut group = c.benchmark_group("medium_compare");

    group.bench_function(BenchmarkId::new("edit_xml", MEDIUM_XML), |b| {
        b.iter(|| {
            let doc = edit_xml::Document::parse_str_with_opts(
                &xml,
                ReadOptions {
                    ..Default::default()
                },
            )
            .unwrap();
            black_box(doc);
        })
    });

    group.bench_function(BenchmarkId::new("xml_tree", MEDIUM_XML), |b| {
        b.iter(|| {
            let doc = xmltree::Element::parse(xml.as_bytes()).unwrap();
            black_box(doc);
        })
    });
    group.bench_function(BenchmarkId::new("roxmltree", MEDIUM_XML), |b| {
        b.iter(|| {
            let doc = roxmltree::Document::parse_with_options(
                xml.as_ref(),
                roxmltree::ParsingOptions {
                    allow_dtd: true,
                    nodes_limit: u32::MAX,
                },
            )
            .unwrap();
            black_box(doc);
        })
    });
    group.finish();
}

fn compare_large(c: &mut Criterion) {
    let path = get_bench_file_path(LARGE_XML);
    let xml = std::fs::read_to_string(path).unwrap();
    let mut group = c.benchmark_group("large_compare");

    group.bench_function(BenchmarkId::new("edit_xml", LARGE_XML), |b| {
        b.iter(|| {
            let doc = edit_xml::Document::parse_str_with_opts(
                &xml,
                ReadOptions {
                    ..Default::default()
                },
            )
            .unwrap();
            black_box(doc);
        })
    });

    group.bench_function(BenchmarkId::new("xml_tree", LARGE_XML), |b| {
        b.iter(|| {
            let doc = xmltree::Element::parse(xml.as_bytes()).unwrap();
            black_box(doc);
        })
    });

    group.bench_function(BenchmarkId::new("roxmltree", LARGE_XML), |b| {
        b.iter(|| {
            let doc = roxmltree::Document::parse_with_options(
                xml.as_ref(),
                roxmltree::ParsingOptions {
                    allow_dtd: true,
                    nodes_limit: u32::MAX,
                },
            )
            .unwrap();
            black_box(doc);
        })
    });
    group.finish();
}
criterion_group! {
    name = tiny;
    config = Criterion::default().sample_size(200);
    targets = compare_tiny
}

criterion_group! {
    name = medium;
    config = Criterion::default().sample_size(100);
    targets = compare_medium
}

criterion_group! {
    name = large;
    config = Criterion::default().sample_size(10);
    targets = compare_large
}

criterion_main!(tiny, medium, large);
