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

"{}
pub mod node;
mod expression;
mod lang;

#[allow(dead_code)]
/// Generated Grammar Specification
pub struct Parser {{
	memos: HashMap<usize, HashMap<String, Box<Option<Vec<Node>>>>>,
}}

#[allow(dead_code)]
impl Parser {{
	pub fn new() -> Self {{
		Parser {{ memos: HashMap::new() }}
	}}

	pub fn parse(&mut self, input: String) -> Result<Option<Node>, Box<dyn Error>> {{
		let mut lang = Lang::new(\"{}\", input);
		if let Some(nodes) = self.{}(&mut lang)? {{
			return Ok(Some(nodes[0].clone()));
		}}
		Ok(None)
	}}

	pub fn parse_file(&mut self, file_path: &str) -> Result<Option<Node>, Box<dyn Error>> {{
		let content = std::fs::read_to_string(file_path)?;
		self.parse(content)
	}}

{}
{}
{}
{}
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
		self.generate_circular_wrapper(),
		self.generate_call_rule(),
		self.generate_rule_functions()?,
		self.generate_meta_rule_functions()?,
		))
	}

	fn generate_circular_wrapper(&self) -> String {
		String::from(

"	fn circular_wrapper(&mut self, lang: &mut Lang, rule_name: String) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let pos = lang.position;

		if let Some(cached_result) = self.memos.get(&pos).and_then(|memo| memo.get(&rule_name)) {
			lang.position = pos;
			return Ok(*cached_result.clone());
		}

		self.memos.entry(pos).or_insert_with(HashMap::new).insert(rule_name.clone(), Box::new(None));

		let mut last_result = None;
		let mut last_pos = pos;

		loop {
			lang.position = pos;

			let result = self.call_rule(&rule_name, lang)?;
			let end_pos = lang.position;

			if end_pos <= last_pos {
				break;
			}

			last_result = result;
			last_pos = end_pos;

			if let Some(memo) = self.memos.get_mut(&pos) {
				memo.insert(rule_name.clone(), Box::new(last_result.clone()));
			}
		}

		lang.position = last_pos;
		Ok(last_result)
	}

"
		)
	}

	fn generate_call_rule(&self) -> String {
		String::from(format!(

"	pub(crate) fn call_rule(&self, rule_name: &str, lang: &mut Lang) -> Result<Option<Vec<Node>>, Box<dyn Error>> {{
		match rule_name {{
{}
			_ => Err(format!(\"Unknown rule: {{}}\", rule_name).into()),
		}}
	}}

",

			{
				let mut cases = String::from("\t\t\t// Regular Rules\n");
				for rule in self.gramspec.rules.keys() {
					cases.push_str(&format!(
						"\t\t\t\"{}\" =>  return self.{}(lang),\n",
						rule,
						rule
					));
				}
				cases.push_str("\t\t\t// Meta Rules\n");
				for rule in self.gramspec.meta_rules.keys() {
					cases.push_str(&format!(
						"\t\t\t\"{}\" =>  return self.{}(lang),\n",
						rule,
						rule
					));
				}
				cases
			}

		))
	}

	fn generate_rule_functions(&self) -> Result<String, Box<dyn Error>> {
		let mut functions = String::from("");
		for rule_name in self.gramspec.rules.keys() {
			let token_expression = self.gramspec.rules.get(rule_name)
				.or_else(|| self.gramspec.meta_rules.get(rule_name))
				.ok_or_else(|| format!("Rule '{}' not found", rule_name))?;

			functions.push_str(format!(

"	pub(crate) fn {}(&self, lang: &mut Lang) -> Result<Option<Vec<Node>>, Box<dyn Error>> {{
		let start_pos = lang.position;
		let mut node = Node::Rule(\"{}\".to_string(), Vec::new(), start_pos);

{}
{}		Ok(None)
	}}

",

			rule_name,
			rule_name,
			{
				if self.gramspec.is_left_circular(rule_name) {
					format!("\t\t// Left recursive\n")
				}else {
					String::from("")
				}
			},
			{
				let mut alternatives = String::from("");
				for alternative in token_expression {
					alternatives.push_str(&format!(

"		if let Some(nodes) = {}.eval(lang, self)? {{
			node.extend(nodes);
			return Ok(Some(vec![node]));
		}}
		lang.position = start_pos;

",

						self.to_conditional(alternative, true)?
					));
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

"	fn {}(&self, lang: &mut Lang) -> Result<Option<Vec<Node>>, Box<dyn Error>> {{
		let start_pos = lang.position;
{}
		let mut nodes = Vec::new();

{}		Ok(None)
	}}

",

			rule_name,
			{
				if self.gramspec.is_left_circular(rule_name) {
					format!("\t\t// Left recursive\n")
				}else {
					String::from("")
				}
			},
			{
				let mut alternatives = String::from("");
				for alternative in token_expression {
					alternatives.push_str(&format!(

"		if let Some(result_nodes) = {}.eval(lang, self)? {{
			nodes.extend(result_nodes);
			return Ok(Some(nodes));
		}} else {{
			lang.position = start_pos;
		}}

",
						self.to_conditional(alternative, true)?
					));
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