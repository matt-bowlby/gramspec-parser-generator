use crate::gramspec::GramSpec;

pub mod expression;
pub mod gramspec;
pub mod node;

mod parser;

pub fn parse(input: &str) -> Result<GramSpec, Box<dyn std::error::Error>> {
    let mut parser = parser::GramspecParser::new();
    let mut nodes = parser.parse(input)?;

    Ok(GramSpec::from(nodes.unwrap()))
}

pub fn parse_file(file_path: &str) -> Result<GramSpec, String> {
    std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))
        .and_then(|content| parse(&content).map_err(|e| format!("Failed to parse file: {}", e)))
}
