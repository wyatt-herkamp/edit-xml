use clap::Parser;
use std::{hint::black_box, path::PathBuf};
#[derive(Debug, Clone, Parser)]
pub struct CLI {
    pub file: PathBuf,
    #[clap(short = 'i', default_value = "1000")]
    pub number_of_iterations: usize,
}
fn main() -> anyhow::Result<()> {
    let args = CLI::parse();

    let string = std::fs::read_to_string(&args.file)?;

    for i in 0..args.number_of_iterations {
        println!("Iteration: {}/{}", i, args.number_of_iterations);
        let document = edit_xml::Document::parse_str(&string).unwrap();
        println!("{}", document.number_of_elements());
        black_box(document);
    }

    Ok(())
}
