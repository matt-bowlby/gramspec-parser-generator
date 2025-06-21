use std::{error::Error};

use crate::gramspec_parser::gramspec::expression::Expression;
use crate::gramspec_parser::gramspec::GramSpec;
// use crate::gramspec_parser::token;
// use crate::gramspec_parser::token::token_type::TokenType;

const USES: &[&str] = &[
	"use std::error::Error;",
	"use crate::parser::node::Node;",
	"use crate::parser::lang::Lang;",
	"use crate::parser::expression::Expression::*;",
];

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

		"{}\n\
		pub mod node;\n\n\
		mod expression;\n\
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

		{
			let mut uses = String::from("");
			for use_statement in USES {
				uses.push_str(&format!("{}\n", use_statement));
			}
			uses
		},
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
				alternatives.push_str(&format!("\t\tif let Some(nodes) = {}.eval(lang)? {{\n", self.to_conditional(alternative, true)?));
				alternatives.push_str("\t\t\tnode.extend(nodes);\n");
				alternatives.push_str("\t\t\treturn Ok(Some(node));\n");
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
				alternatives.push_str(&format!("\t\tif let Some(result_node) = {} {{\n", self.to_conditional(alternative, true)?));
				alternatives.push_str("\t\t\treturn Ok(Some(result_node));\n");
				alternatives.push_str("\t\t}\n");
				alternatives.push_str("\t\tlang.position = start_pos;\n\n");
			}
			alternatives
		},

		))
	}

	fn to_conditional(&self, expression: &Expression, is_first: bool) -> Result<String, Box<dyn Error>> {
		if is_first {
			match expression {
				Expression::RuleName(name) => Ok(format!("RuleName(\"{}\")", name.value)),
				Expression::Keyword(keyword) => Ok(format!("Keyword(\"{}\")", keyword.value)),
				Expression::RegexLiteral(regex) => Ok(format!("RegexLiteral(r\"{}\")", regex.value)),
				Expression::StringLiteral(string) => Ok(format!("StringLiteral(\"{}\")", string.value)),
				Expression::Or(left, right) => Ok(format!("Or({}, {})", self.to_conditional(left, false)?, self.to_conditional(right, false)?)),
				Expression::And(left, right) => Ok(format!("And({}, {})", self.to_conditional(left, false)?, self.to_conditional(right, false)?)),
				Expression::DelimitRepeatOne(left, right) => Ok(format!("DelimitRepeatOne({}, {})", self.to_conditional(left, false)?, self.to_conditional(right, false)?)),
				Expression::DelimitRepeatZero(left, right) => Ok(format!("DelimitRepeatZero({}, {})", self.to_conditional(left, false)?, self.to_conditional(right, false)?)),
				Expression::Optional(expr) => Ok(format!("Optional({})", self.to_conditional(expr, false)?)),
				_ => Err(format!("Unsupported expression type in rule generation").into()),
			}
		} else {
			match expression {
				Expression::RuleName(name) => Ok(format!("Box::new(RuleName(\"{}\"))", name.value)),
				Expression::Keyword(keyword) => Ok(format!("Box::new(Keyword(\"{}\"))", keyword.value)),
				Expression::RegexLiteral(regex) => Ok(format!("Box::new(RegexLiteral(r\"{}\"))", regex.value)),
				Expression::StringLiteral(string) => Ok(format!("Box::new(StringLiteral(\"{}\"))", string.value)),
				Expression::Or(left, right) => Ok(format!("Box::new(Or({}, {}))", self.to_conditional(left, false)?, self.to_conditional(right, false)?)),
				Expression::And(left, right) => Ok(format!("Box::new(And({}, {}))", self.to_conditional(left, false)?, self.to_conditional(right, false)?)),
				Expression::DelimitRepeatOne(left, right) => Ok(format!("Box::new(DelimitRepeatOne({}, {}))", self.to_conditional(left, false)?, self.to_conditional(right, false)?)),
				Expression::DelimitRepeatZero(left, right) => Ok(format!("Box::new(DelimitRepeatZero({}, {}))", self.to_conditional(left, false)?, self.to_conditional(right, false)?)),
				Expression::Optional(expr) => Ok(format!("Box::new(Optional({}))", self.to_conditional(expr, false)?)),
				_ => Err(format!("Unsupported expression type in rule generation").into()),
			}
		}
	}
}