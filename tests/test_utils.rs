#![allow(dead_code)]
use std::path::PathBuf;

use edit_xml::ReadOptions;
use itertools::Itertools;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize the logger for the tests
pub fn setup_logger() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let stdout_log = tracing_subscriber::fmt::layer().pretty();
        tracing_subscriber::registry().with(stdout_log).init();
    });
    info!("Logger initialized");
    debug!("Logger initialized");
}
pub fn test_dir() -> std::path::PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")
}

pub fn iter_read_options() -> impl Iterator<Item = ReadOptions> {
    let empty_text_node_opts = [true, false];
    let trim_text = [true, false];
    let ignore_whitespace_only = [true, false];
    let require_decl = [true, false];
    let opts = [
        empty_text_node_opts,
        trim_text,
        ignore_whitespace_only,
        require_decl,
    ];
    opts.into_iter().multi_cartesian_product().map(|raw| {
        let mut read_options = ReadOptions::default();
        read_options.empty_text_node = raw[0];
        read_options.trim_text = raw[1];
        read_options.ignore_whitespace_only = raw[2];
        read_options.require_decl = raw[3];
        read_options
    })
}
