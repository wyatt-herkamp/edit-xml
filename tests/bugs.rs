use std::fs::read_to_string;

mod test_utils;
#[test]
fn oslash() -> anyhow::Result<()> {
    test_utils::setup_logger();
    let file = read_to_string(
        test_utils::test_dir()
            .join("bugs")
            .join("oslash")
            .join("oslash.xml"),
    )
    .unwrap();

    let doc = edit_xml::Document::parse_str_with_opts(
        &file,
        edit_xml::ReadOptions {
            require_decl: false,
            ..Default::default()
        },
    )
    .unwrap();

    println!("Parse Successful");
    Ok(())
}
