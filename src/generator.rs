use std::{error::Error};

use crate::gramspec_parser::gramspec::expression::Expression;
use crate::gramspec_parser::gramspec::GramSpec;
// use crate::gramspec_parser::token;
// use crate::gramspec_parser::token::token_type::TokenType;

pub struct Generator {
	gramspec: GramSpec,
}

impl Generator {
	pub fn new(gramspec: GramSpec) -> Self {
		Generator { gramspec }
	}

	pub fn generate(&self, language_name: String) -> Result<String, Box<dyn Error>> {
		let trimmed_language_name = language_name.trim().trim_matches(' ').to_string();
		Ok(format!(

		"use std::error::Error;\n\n\
		use crate::parser::node::Node;\n\
		use crate::parser::lang::Lang;\n\n\
		pub mod node;\n\
		mod lang;\n\n\
		/// Generated Grammar Specification\n\
		struct {} {{ }}\n\n\
		impl {} {{\n\n\
			\tpub fn new() -> Self {{\n\
				\t\t{} {{ }}\n\
			\t}}\n\n\
			\tpub fn parse(&mut self, input: String) -> Result<Option<Node>, Box<dyn Error>> {{\n\
				\t\tlet mut lang = Lang::new(\"{}\", input);\n\
				\t\tself.{}(&mut lang)?;\n\
				\t\tOk(None)\n\
			\t}}\n\n\
			{}\
			{}\
		}}",

		trimmed_language_name,
		trimmed_language_name,
		trimmed_language_name, language_name,
		self.gramspec.config.entry_rule,
		{
			let mut functions = String::from("");
			for rule in self.gramspec.rules.keys() {
				functions.push_str(&self.generate_rule_function(rule)?);
			}
			functions
		},
		{
			let mut meta_functions = String::from("");
			for meta_rule in self.gramspec.meta_rules.keys() {
				meta_functions.push_str(&self.generate_meta_rule_function(meta_rule)?);
			}
			meta_functions
		}

		))
	}


	fn generate_rule_function(
		&self,
		rule_name: &String
	) -> Result<String, Box<dyn Error>> {

		let token_expression = self.gramspec.rules.get(rule_name)
			.or_else(|| self.gramspec.meta_rules.get(rule_name))
			.ok_or_else(|| format!("Rule '{}' not found", rule_name))?;

		Ok(format!(

		"\tfn {}(&mut self, lang: &mut Lang) -> Result<Option<Node>, Box<dyn Error>> {{\n\
			\t\tlet mut node = Node::Rule(\"{}\".to_string(), Vec::new());\n\n\
			{}\
			\t\tOk(None)\n\
		\t}}\n\n",

		rule_name,
		rule_name,
		{
			let mut alternatives = String::from("\t\tlet start_pos = lang.position;\n");
			for alternative in token_expression {
				alternatives.push_str(&format!("\t\tif let Some(result_node) = {} {{\n", self.to_conditional(alternative)?));
				alternatives.push_str("\t\t\treturn Ok(Some(result_node));\n");
				alternatives.push_str("\t\t}\n");
				alternatives.push_str("\t\tlang.position = start_pos;\n\n");
			}
			alternatives
		},

		))
	}

	fn generate_meta_rule_function(
		&self,
		rule_name: &String
	) -> Result<String, Box<dyn Error>> {

		let token_expression = self.gramspec.rules.get(rule_name)
			.or_else(|| self.gramspec.meta_rules.get(rule_name))
			.ok_or_else(|| format!("meta-rule '{}' not found", rule_name))?;

		Ok(format!(

		"\tfn {}(&mut self, lang: &mut Lang) -> Result<Vec<Node>, Box<dyn Error>> {{\n\
			\t\tlet mut nodes = Vec::new();\n\n\
			{}\
			\t\tOk(nodes)\n\
		\t}}\n\n",

		rule_name,
		{
			let mut alternatives = String::from("\t\tlet start_pos = lang.position;\n");
			for alternative in token_expression {
				alternatives.push_str(&format!("\t\tif let Some(result_node) = {} {{\n", self.to_conditional(alternative)?));
				alternatives.push_str("\t\t\treturn Ok(Some(result_node));\n");
				alternatives.push_str("\t\t}\n");
				alternatives.push_str("\t\tlang.position = start_pos;\n\n");
			}
			alternatives
		},

		))
	}

	fn to_conditional(&self, expression: &Expression) -> Result<String, Box<dyn Error>> {
		match expression {
			Expression::RuleName(name) => Ok(format!("self.{}(lang, node)?", name.value)),
			Expression::Keyword(keyword) => Ok(format!("lang.expect_keyword(\"{}\", node)?", keyword.value)),
			Expression::RegexLiteral(regex) => Ok(format!("lang.expect_regex(\"{}\", node)?", regex.value)),
			Expression::StringLiteral(string) => Ok(format!("lang.expect_string(\"{}\", node)?", string.value)),
			Expression::Or(left, right) => Ok(format!("lang.expect_or({}, {}, node)?", self.to_conditional(left)?, self.to_conditional(right)?)),
			Expression::And(left, right) => Ok(format!("lang.expect_and({}, {}, node)?", self.to_conditional(left)?, self.to_conditional(right)?)),
			Expression::DelimitRepeatOne(left, right) => Ok(format!("lang.expect_delimit_repeat_one({}, {}, node)?", self.to_conditional(left)?, self.to_conditional(right)?)),
			Expression::DelimitRepeatZero(left, right) => Ok(format!("lang.expect_delimit_repeat_zero({}, {}, node)?", self.to_conditional(left)?, self.to_conditional(right)?)),
			Expression::Optional(expr) => Ok(format!("lang.expect_optional({}, node)?", self.to_conditional(expr)?)),
			_ => Err(format!("Unsupported expression type in rule generation").into()),
		}
	}
}