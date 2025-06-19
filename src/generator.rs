use std::{error::Error};

use crate::gramspec_parser::gramspec::GramSpec;

pub struct Generator {
	gramspec: GramSpec,
}

impl Generator {
	pub fn new(gramspec: GramSpec) -> Self {
		Generator { gramspec }
	}

	pub fn generate(&self, language_name: String) -> Result<String, Box<dyn Error>> {
		let mut output = String::new();

		output.push_str("use std::error::Error;\n\n");

		output.push_str("use crate::parser::lang::Lang;\n\n");

		output.push_str("mod lang;\n\n");


		output.push_str("/// Generated Grammar Specification\n");
		// Open brace for the struct definition
		output.push_str(format!("struct {} {{\n", language_name.trim_matches(' ')).as_str());
		output.push_str("    lang: Lang,\n");
		// Closing brace for the struct definition
		output.push_str("}\n\n");
		// Implement the struct
		output.push_str(format!("impl {} {{\n\n", language_name.trim_matches(' ')).as_str());

		output.push_str(format!("    pub fn new() -> Self {{\n").as_str());
		output.push_str(format!("        {} {{ lang: Lang::new(\"{}\", \"\".to_string()) }}\n", language_name, language_name).as_str());

		output.push_str("    }\n\n");

		// Todo: Replace unit type with the actual return type of the parse function, a Node
		output.push_str("    pub fn parse(&mut self, input: String) -> Result<Option<()>, Box<dyn Error>> {\n");

		let entry_rule = &self.gramspec.config.entry_rule;
		if self.gramspec.rules.get(entry_rule).is_none() {
			return Err(
				format!("Entry rule '{}' not found in grammar specification", entry_rule).into()
			);
		}

		output.push_str(format!("        self.lang.set_content(input);\n").as_str());

		output.push_str(format!("        self.{}()?;\n", entry_rule).as_str());

		output.push_str("        Ok(None)\n");

		output.push_str("    }\n\n");

		output.push_str(format!("    fn {}(&mut self) -> Result<Option<()>, Box<dyn Error>> {{\n", entry_rule).as_str());

		output.push_str("        Ok(None)\n");

		output.push_str("    }\n\n");

		// Closing brace for the struct implementation
		output.push_str("}\n\n");

		Ok(output)
	}
}