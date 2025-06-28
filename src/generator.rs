use std::{error::Error};

use crate::gramspec_parser::gramspec::expression::Expression;
use crate::gramspec_parser::gramspec::GramSpec;
// use crate::gramspec_parser::token;
// use crate::gramspec_parser::token::token_type::TokenType;

const USES: &[&str] = &[
	"use std::error::Error;",
	"use std::collections::HashMap;",
	"\n",
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
		Ok(format!(

		"{}\n\
		pub mod node;\n\n\
		mod expression;\n\
		mod lang;\n\n\
		#[allow(dead_code)]\n\
		/// Generated Grammar Specification\n\
		pub struct Parser {{\n\
			\tmemos: HashMap<String, HashMap<String, Option<String>>>,\n\
		 }}\n\n\
		#[allow(dead_code)]\n\
		impl Parser {{\n\n\
			\tpub fn new() -> Self {{\n\
				\t\tParser {{ memos: HashMap::new() }}\n\
			\t}}\n\n\
			\tpub fn parse(&mut self, input: String) -> Result<Option<Node>, Box<dyn Error>> {{\n\
				\t\tlet mut lang = Lang::new(\"{}\", input);\n\
				\t\tif let Some(nodes) = self.{}(&mut lang)? {{\n\
					\t\t\treturn Ok(Some(nodes[0].clone()));\n\
				\t\t}}\n\
				\t\tOk(None)\n\
			\t}}\n\n\
			\tpub fn parse_file(&mut self, file_path: &str) -> Result<Option<Node>, Box<dyn Error>> {{\n\
				\t\tlet content = std::fs::read_to_string(file_path)?;\n\
				\t\tself.parse(content)\n\
			\t}}\n\n\
			{}\
			{}\
			{}\
		}}",

		// Use Statements
		{
			let mut uses = String::from("");
			for use_statement in USES {
				uses.push_str(&format!("{}\n", use_statement));
			}
			uses
		},
		language_name,
		self.gramspec.config.entry_rule,
		self.generate_call_rule(),
		self.generate_rule_functions()?,
		self.generate_meta_rule_functions()?,
		))
	}

	fn generate_call_rule(&self) -> String {
		String::from(format!(

		"\tpub(crate) fn call_rule(&self, rule_name: &str, lang: &mut Lang) -> Result<Option<Vec<Node>>, Box<dyn Error>> {{\n\
			\t\tmatch rule_name {{\n\
			{}\
			\t\t\t_ => Err(format!(\"Unknown rule: {{}}\", rule_name).into()),\n\
			\t\t}}\n\
		\t}}\n\n\
		",

		{
			let mut cases = String::from("\t\t\t// Regular Rules\n");
			for rule in self.gramspec.rules.keys() {
				cases.push_str(&format!(
					"\t\t\t\"{}\" =>  return self.{}(lang),\n", rule, rule
				));
			}
			cases.push_str("\n\t\t\t// Meta Rules\n");
			for rule in self.gramspec.meta_rules.keys() {
				cases.push_str(&format!(
					"\t\t\t\"{}\" =>  return self.{}(lang),\n", rule, rule
				));
			}
			cases
		}

		))
	}

	fn generate_rule_functions(&self) -> Result<String, Box<dyn Error>> {
		let mut functions = String::from("");
		for rule_name in self.gramspec.rules.keys() {
			println!("Generating function for rule: {}", rule_name);
			let token_expression = self.gramspec.rules.get(rule_name)
				.or_else(|| self.gramspec.meta_rules.get(rule_name))
				.ok_or_else(|| format!("Rule '{}' not found", rule_name))?;

			functions.push_str(format!(

			"\tpub(crate) fn {}(&self, lang: &mut Lang) -> Result<Option<Vec<Node>>, Box<dyn Error>> {{\n\
				{}\
				\t\tlet mut node = Node::Rule(\"{}\".to_string(), Vec::new());\n\n\
				{}\
				\t\tOk(None)\n\
			\t}}\n\n",

			rule_name,
			{
				if self.gramspec.is_left_circular(rule_name) {
					format!("\t\t// Left recursive\n")
				}else {
					String::from("")
				}
			},
			rule_name,
			{
				let mut alternatives = String::from("\t\tlet start_pos = lang.position;\n");
				for alternative in token_expression {
					alternatives.push_str(&format!("\t\tif let Some(nodes) = {}.eval(lang, self)? {{\n", self.to_conditional(alternative, true)?));
					alternatives.push_str("\t\t\tnode.extend(nodes);\n");
					alternatives.push_str("\t\t\treturn Ok(Some(vec![node]));\n");
					alternatives.push_str("\t\t}\n");
					alternatives.push_str("\t\tlang.position = start_pos;\n\n");
				}
				alternatives
			},

			).as_str());
		}
		Ok(functions)
	}

	fn generate_meta_rule_functions(&self) -> Result<String, Box<dyn Error>> {
		let mut functions = String::from("");
		for rule_name in self.gramspec.meta_rules.keys() {
			let token_expression = self.gramspec.rules.get(rule_name)
				.or_else(|| self.gramspec.meta_rules.get(rule_name))
				.ok_or_else(|| format!("meta-rule '{}' not found", rule_name))?;

			functions.push_str(&format!(

			"\tfn {}(&self, lang: &mut Lang) -> Result<Option<Vec<Node>>, Box<dyn Error>> {{\n\
				{}\
				\t\tlet mut nodes = Vec::new();\n\n\
				{}\
				\t\tOk(None)\n\
			\t}}\n\n",

			rule_name,
			{
				if self.gramspec.is_left_circular(rule_name) {
					format!("\t\t// Left recursive\n")
				}else {
					String::from("")
				}
			},
			{
				let mut alternatives = String::from("\t\tlet start_pos = lang.position;\n");
				for alternative in token_expression {
					alternatives.push_str(&format!("\t\tif let Some(result_nodes) = {}.eval(lang, self)? {{\n", self.to_conditional(alternative, true)?));
					alternatives.push_str("\t\t\tnodes.extend(result_nodes);\n");
					alternatives.push_str("\t\t\treturn Ok(Some(nodes));\n");
					alternatives.push_str("\t\t} else {\n");
					alternatives.push_str("\t\t\tlang.position = start_pos;\n");
					alternatives.push_str("\t\t}\n");
				}
				alternatives
			},
			));
		}
		Ok(functions)
	}

	fn to_conditional(&self, expression: &Expression, is_first: bool) -> Result<String, Box<dyn Error>> {
		if is_first {
			match expression {
				Expression::RuleName(name) => Ok(format!("Rule(\"{}\")", name.value)),
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
				Expression::RuleName(name) => Ok(format!("Box::new(Rule(\"{}\"))", name.value)),
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