mod gramspec_parser;
mod generator;
// mod parser;

use gramspec_parser::parser::Parser;
use generator::Generator;
use std::fs;

fn main() {
	let generate = true;

	if generate {
		// Read the grammar specification and code files
		let gramspec = fs::read_to_string("test_files/test.grm").unwrap();
		// let code = fs::read_to_string("test_files/test.txt").unwrap();
		// Tokenize the grammar specification
		let mut parser = Parser::new(gramspec);
		// Tokenize the input string
		let gramspec = parser.parse().unwrap_or_else(|err| {
			eprintln!("Error parsing grammar specification: {}", err);
			std::process::exit(1);
		});
		// Generate the parser code from the grammar specification
		let generator = Generator::new(gramspec);
		generator.generate("./src/parser.rs", "PlainTalkParser", "    ").unwrap();
	} else {
		// print!("{}", parser::Parser::new()
		// 	.parse_file("test_files/test.txt")
		// 	.unwrap() // Super unsafe, but for testing purposes
		// 	.unwrap() // Same as above
		// 	.pretty_print(0));
	}
}
