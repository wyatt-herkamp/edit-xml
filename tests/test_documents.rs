mod test_utils;
use test_utils::{document_validation::*, setup_logger};
#[test]
fn nodes() {
    setup_logger();
    execute_test_with_all_options("nodes.xml").unwrap()
}

#[test]
fn doc() {
    setup_logger();
    execute_test_with_all_options("doc.xml").unwrap();
}
#[test]
fn encoding1() {
    setup_logger();
    execute_test_with_all_options("encoding1.xml").unwrap();
}
#[test]
fn encoding2() {
    setup_logger();
    execute_test_with_all_options("encoding2.xml").unwrap();
}
