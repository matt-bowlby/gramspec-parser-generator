mod generator;
mod gramspec_parser;
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
        // let code = fs::read_to_string("temp/test.txt").unwrap();
        // Tokenize the grammar specification
        let mut parser = Parser::new(gramspec);
        // Tokenize the input string
        let gramspec = parser.parse().unwrap_or_else(|err| {
            eprintln!("Error parsing grammar specification: {}", err);
            std::process::exit(1);
        });
        // Generate the parser code from the grammar specification
        let generator = Generator::new(gramspec);
        generator
            .generate("./src/parser.rs", "GramspecParser", "    ")
            .unwrap();
    } else {
        let result = GramspecParser::new()
            // .enable_debug()
            .parse_file("temp/gramspec.grm");

        match result {
            Ok(result) => result.pretty_print(),
            Err(err) => {
                eprintln!("Error parsing grammar specification: {}", err);
                std::process::exit(1);
            }
        }
    }
}
