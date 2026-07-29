mod gramspec_parser;
mod generator;
mod parser;

use gramspec_parser::parser::Parser;
use parser::GramspecParser;

use generator::Generator;
use std::fs;

fn main() {
    let generate = false;

    if generate {
        // Read the grammar specification and code files
        let gramspec = fs::read_to_string("temp/gramspec.grm").unwrap();
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
        generator.generate("./src/parser.rs", "GramspecParser", "    ").unwrap();
    } else {
        match GramspecParser::new()
            // .enable_debug()
            .parse_file("temp/gramspec.grm") {
                Ok(node) => node.pretty_print(),
                Err(message)=> eprintln!("Error parsing file: {:?}", message),
        }
    }
}
