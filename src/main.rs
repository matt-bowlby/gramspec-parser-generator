mod gramspec_parser;
mod generator;
mod parser;

use gramspec_parser::parser::Parser;
use generator::Generator;
use std::fs;

fn main() {
	// Read the grammar specification and code files
	let gramspec = fs::read_to_string("test_files/test.grm").unwrap();
	// let code = fs::read_to_string("test_files/test.txt").unwrap();
	// Tokenize the grammar specification
	let mut parser = Parser::new(gramspec);
	// Tokenize the input string
	let gramspec = parser.parse().unwrap();
	// Generate the parser code from the grammar specification
	let generator = Generator::new(gramspec);
	let output = generator.generate("TestLanguage".to_string()).unwrap();
	// Write the generated code to a file
	fs::write("./src/parser.rs", output).unwrap();
}
