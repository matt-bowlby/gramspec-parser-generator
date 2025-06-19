mod gramspec_parser;
use gramspec_parser::parser::Parser;
use std::fs;

fn main() {
	// Read the grammar specification and code files
	let gramspec = fs::read_to_string("test_files/test.grm").unwrap();
	// let code = fs::read_to_string("test_files/test.txt").unwrap();
	// Tokenize the grammar specification
	let mut parser = Parser::new(gramspec);
	// Tokenize the input string
	let gram_spec = parser.parse().unwrap();

	for (rule_name, expression) in gram_spec.rules {
		println!("Rule: {}", rule_name);
		println!("Expression: {:?}", expression);
	}
}
