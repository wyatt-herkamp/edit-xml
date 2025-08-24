#![allow(dead_code)]
use std::path::PathBuf;

use anyhow::Context;
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
pub fn documents_dir() -> std::path::PathBuf {
    test_dir().join("documents")
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
    opts.into_iter()
        .multi_cartesian_product()
        .map(|raw| ReadOptions {
            empty_text_node: raw[0],
            trim_text: raw[1],
            ignore_whitespace_only: raw[2],
            require_decl: raw[3],
            // I don't like updating all the tests rn
            normalize_attribute_value_space: true,
            ..Default::default()
        })
}
/// Create a file name for the read options
///
///
/// - Empty Text Node: etn
/// - Trim Text: trim
/// - Ignore Whitespace Only: ignws
/// - Require Declaration: reqdecl
pub fn create_read_options_file_name(options: &ReadOptions) -> String {
    let mut params = Vec::new();
    if options.empty_text_node {
        params.push("etn");
    }
    if options.trim_text {
        params.push("trim");
    }
    if options.ignore_whitespace_only {
        params.push("ignws");
    }
    if options.require_decl {
        params.push("reqdecl");
    }
    if params.is_empty() {
        return "everything_disabled".to_owned();
    }
    params.join("_")
}

pub fn should_update_expected() -> anyhow::Result<bool> {
    match std::env::var("UPDATE_EXPECTED") {
        Ok(ok) => Ok(ok
            .to_ascii_lowercase()
            .parse::<bool>()
            .context("Failed to parse UPDATE_EXPECTED. Should be true or false")?),
        Err(err) => match err {
            std::env::VarError::NotPresent => Ok(false),
            _ => Err(err.into()),
        },
    }
}

pub mod document_validation {
    use std::process::exit;

    use super::{documents_dir, should_update_expected};
    use edit_xml::{Document, DocumentBreakdown, EditXMLError, NodeBreakdown, ReadOptions};
    use serde::{Deserialize, Serialize};
    use tracing::{debug, error, info, warn};
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TestError {
        pub message: String,
    }
    impl From<EditXMLError> for TestError {
        fn from(err: EditXMLError) -> Self {
            error!("Decide on how to represent errors: {:?}", err);
            Self {
                message: "todo".to_owned(),
            }
        }
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum TestDocumentResult {
        Success(DocumentBreakdown),
        Error(TestError),
    }
    impl TestDocumentResult {
        pub fn read_expected(file: &str) -> anyhow::Result<Option<Self>> {
            let file_path = documents_dir().join(file);
            debug!("Reading expected from: {:?}", file_path);
            if !file_path.exists() {
                return Ok(None);
            }
            let file = std::fs::read_to_string(file_path)?;
            let expected = serde_json::from_str(&file)?;
            Ok(Some(expected))
        }
        pub fn write(&self, file: &str) -> anyhow::Result<()> {
            let file_path = documents_dir().join(file);
            if !file_path.exists() {
                let parent = file_path.parent().unwrap();
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            let file = std::fs::File::create(file_path)?;
            serde_json::to_writer_pretty(file, self)?;
            Ok(())
        }
        pub fn update_from_result(
            result: Result<Document, EditXMLError>,
            path: &str,
        ) -> anyhow::Result<Self> {
            match result {
                Ok(ok) => Self::update_breakdown(&ok, path),
                Err(err) => Self::update_error(err, path),
            }
        }
        pub fn update_breakdown(doc: &Document, path: &str) -> anyhow::Result<Self> {
            let result = TestDocumentResult::Success(doc.breakdown());
            result.write(path)?;
            Ok(result)
        }
        pub fn update_error(error: impl Into<TestError>, path: &str) -> anyhow::Result<Self> {
            let result = TestDocumentResult::Error(error.into());
            result.write(path)?;
            Ok(result)
        }
    }
    #[allow(
        clippy::large_enum_variant,
        reason = "This is for testing. Size is not a concern"
    )]
    pub enum BreakdownComparisonResult {
        Match,
        MismatchedSize {
            expected: usize,
            actual: usize,
        },
        MismatchedBreakdown {
            expected: NodeBreakdown,
            actual: NodeBreakdown,
        },
    }
    fn expected_breakdown_file(xml_file: &str, read_options: &ReadOptions) -> String {
        let results_folder = xml_file.trim_end_matches(".xml").to_owned();
        let read_options_file_name = super::create_read_options_file_name(read_options);
        format!("{results_folder}/{read_options_file_name}.json")
    }
    pub fn execute_test_with_all_options(xml_file: &'static str) -> anyhow::Result<()> {
        for read_options in super::iter_read_options() {
            execute_test_with_read_options(xml_file, read_options)?;
        }
        Ok(())
    }
    pub fn execute_test_with_read_options(
        xml_file: &'static str,
        read_options: ReadOptions,
    ) -> anyhow::Result<()> {
        let expected_file_path = expected_breakdown_file(xml_file, &read_options);

        let expected_file = TestDocumentResult::read_expected(&expected_file_path)?;
        let should_update = should_update_expected()?;
        let doc_path = documents_dir().join(xml_file);
        if !doc_path.exists() {
            anyhow::bail!("File not found: {:?}", doc_path);
        }
        let doc = Document::parse_file_with_opts(doc_path, read_options.clone());
        let Some(expected) = expected_file else {
            if should_update {
                info!("Expected breakdown file not found. Updating breakdowns");
                TestDocumentResult::update_from_result(doc, &expected_file_path)?;
                info!("Breakdowns updated. Exiting...");
                return Ok(());
            } else {
                anyhow::bail!("Expected breakdown file not found: {}", expected_file_path);
            }
        };
        match doc {
            Ok(ok) => {
                let result = got_success(expected, &ok, should_update, &expected_file_path)?;
                if result {
                    warn!("Breakdowns were updated. Please run the test again.");
                    return Ok(());
                }
                write_then_read_again(ok, read_options)?;
            }
            Err(err) => {
                got_error(expected, err, should_update, &expected_file_path)?;
            }
        }
        Ok(())
    }
    fn got_success(
        expected: TestDocumentResult,
        doc: &Document,
        should_update: bool,
        expected_file_path: &str,
    ) -> anyhow::Result<bool> {
        let actual = doc.breakdown();
        let TestDocumentResult::Success(expected) = expected else {
            panic!("Expected error, but got success");
        };
        let comparison_result = assert_breakdowns(&expected.root_elements, actual.root_elements);
        match comparison_result {
            BreakdownComparisonResult::MismatchedSize { expected, actual } => {
                if should_update {
                    TestDocumentResult::update_breakdown(doc, expected_file_path)?;
                    info!("Breakdowns updated. ");
                    Ok(true)
                } else {
                    panic!("Breakdowns do not match\nExpected: {expected:#?}\nActual: {actual:#?}");
                }
            }
            BreakdownComparisonResult::MismatchedBreakdown { expected, actual } => {
                if should_update {
                    TestDocumentResult::update_breakdown(doc, expected_file_path)?;
                    info!("Breakdowns updated. ");
                    Ok(true)
                } else {
                    panic!("Breakdowns do not match\nExpected: {expected:#?}\nActual: {actual:#?}");
                }
            }
            BreakdownComparisonResult::Match => {
                println!("Breakdowns match");
                Ok(false)
            }
        }
    }
    fn got_error(
        expected: TestDocumentResult,
        actual: EditXMLError,
        should_update: bool,
        expected_file_path: &str,
    ) -> anyhow::Result<()> {
        let TestDocumentResult::Error(expected) = expected else {
            panic!("Expected success, but got error");
        };
        let actual = TestError::from(actual);
        if expected != actual {
            if should_update {
                TestDocumentResult::update_error(actual, expected_file_path)?;
                eprintln!("Breakdowns updated. Exiting...");
                exit(1);
            } else {
                panic!("Breakdowns do not match\nExpected: {expected:#?}\nActual: {actual:#?}");
            }
        }
        Ok(())
    }
    pub fn write_then_read_again(input: Document, read_options: ReadOptions) -> anyhow::Result<()> {
        let expected_breakdowns = input.breakdown();
        let result = input.write_str()?;

        let doc = Document::parse_str_with_opts(&result, read_options)
            .expect("Failed to parse the written document");

        let actual_breakdowns = doc.breakdown();

        let comparison_result = assert_breakdowns(
            &expected_breakdowns.root_elements,
            actual_breakdowns.root_elements,
        );

        match comparison_result {
            BreakdownComparisonResult::MismatchedSize { expected, actual } => {
                panic!(
                    "Breakdowns do not match(Write Then Read)\nExpected: {expected:#?}\nActual: {actual:#?}"
                );
            }
            BreakdownComparisonResult::MismatchedBreakdown { expected, actual } => {
                panic!(
                    "Breakdowns do not match(Write Then Read)\nExpected: {expected:#?}\nActual: {actual:#?}"
                );
            }
            BreakdownComparisonResult::Match => {
                println!("Rewrite Breakdowns match(Write Then Read)");
                Ok(())
            }
        }
    }

    fn assert_breakdowns(
        expected: &[NodeBreakdown],
        actual: Vec<NodeBreakdown>,
    ) -> BreakdownComparisonResult {
        if expected.len() != actual.len() {
            return BreakdownComparisonResult::MismatchedSize {
                expected: expected.len(),
                actual: actual.len(),
            };
        }

        for (expected, actual) in expected.iter().zip(actual.iter()) {
            if expected != actual {
                return BreakdownComparisonResult::MismatchedBreakdown {
                    expected: expected.clone(),
                    actual: actual.clone(),
                };
            }
        }
        BreakdownComparisonResult::Match
    }
}
