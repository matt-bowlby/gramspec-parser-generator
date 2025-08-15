use std::{error::Error};

use crate::gramspec_parser::gramspec::expression::Expression;
use crate::gramspec_parser::gramspec::GramSpec;
// use crate::gramspec_parser::token;
// use crate::gramspec_parser::token::token_type::TokenType;

const USES: &[&str] = &[
	"mod expression;",
	"mod node;",
	"",
	"use std::error::Error;",
	"use std::collections::HashMap;",
	"use regex::Regex;",
	"",
	"use expression::Expression::{self, *};",
	"use node::Node;",
];

pub struct Generator {
	gramspec: GramSpec,
}

impl Generator {
	pub fn new(gramspec: GramSpec) -> Self {
		Generator { gramspec }
	}

	pub fn generate(&self) -> Result<String, Box<dyn Error>> {
		Ok(format!(

"{}
const KEYWORDS: &[(&str, &str)] = &[
	(\"ENDMARKER\", \"0\"),
];

#[allow(dead_code)]
/// Generated Grammar Specification
pub struct Parser {{
	pub position: usize,

	content: String,
	memos: HashMap<usize, HashMap<String, Box<Option<Vec<Node>>>>>,
}}

#[allow(dead_code)]
impl Parser {{
	pub fn new() -> Self {{
		Parser {{ content: String::new(), position: 0, memos: HashMap::new() }}
	}}

	pub fn parse(&mut self, input: String) -> Result<Option<Node>, Box<dyn Error>> {{
		self.position = 0;
		self.content = input;
		if let Some(nodes) = self.{}()? {{
			return Ok(Some(nodes[0].clone()));
		}}
		Ok(None)
	}}

	pub fn parse_file(&mut self, file_path: &str) -> Result<Option<Node>, Box<dyn Error>> {{
		let content = std::fs::read_to_string(file_path)?;
		self.parse(content)
	}}

	fn circular_wrapper(&mut self, rule_name: String) -> Result<Option<Vec<Node>>, Box<dyn Error>> {{
		let pos = self.position;

		if let Some(cached_result) = self.memos.get(&pos).and_then(|memo| memo.get(&rule_name)) {{
			self.position = pos;
			return Ok(*cached_result.clone());
		}}

		self.memos.entry(pos).or_insert_with(HashMap::new).insert(rule_name.clone(), Box::new(None));

		let mut last_result = None;
		let mut last_pos = pos;

		loop {{
			self.position = pos;

			let result = self.call_rule(&rule_name, false)?;
			let end_pos = self.position;

			if end_pos <= last_pos {{
				break;
			}}

			last_result = result;
			last_pos = end_pos;

			if let Some(memo) = self.memos.get_mut(&pos) {{
				memo.insert(rule_name.clone(), Box::new(last_result.clone()));
			}}
		}}

		self.position = last_pos;
		Ok(last_result)
	}}

	fn expect_string(&mut self, string: &str) -> Result<Option<Vec<Node>>, Box<dyn Error>> {{
		let start_pos = self.position;
		loop {{
			if self.content[self.position..].starts_with(string) {{
				self.position += string.len();
				return Ok(Some(vec![Node::String(string.to_string(), start_pos)]));
			}} else {{
				if let Some(whitespace) = Regex::new(r\"^\\s+\").unwrap().find(&self.content[self.position..]) {{
					self.position += whitespace.end();
				}} else {{
					break;
				}}
			}}
		}}
		self.position = start_pos;
		Ok(None)
	}}

	fn expect_regex(&mut self, regex: &str) -> Result<Option<Vec<Node>>, Box<dyn Error>> {{
		let start_pos = self.position;
		loop {{
			if let Some(captures) = Regex::new(regex).unwrap().captures(&self.content[self.position..]) {{
				self.position += captures.get(0).unwrap().end();
				return Ok(Some(vec![Node::String(captures.get(0).unwrap().as_str().to_string(), start_pos)]));
			}} else {{
				if let Some(whitespace) = Regex::new(r\"^\\s+\").unwrap().find(&self.content[self.position..]) {{
					self.position += whitespace.end();
				}} else {{
					break;
				}}
			}}
		}}
		self.position = start_pos;
		Ok(None)
	}}

	fn get_keywords_map(&self) -> HashMap<String, String> {{
		KEYWORDS.iter()
			.map(|(k, v)| (k.to_string(), v.to_string()))
			.collect()
	}}

	fn expect_keyword(&mut self, keyword: &str) -> Result<Option<Vec<Node>>, Box<dyn Error>> {{
		let start_pos = self.position;
		let keyword_value = self.get_keywords_map().get(keyword)
			.ok_or_else(|| format!(\"Unknown keyword: {{}}\", keyword))?
			.to_owned();
		if self.content[self.position..].starts_with(&keyword_value) {{
			self.position += keyword_value.len();
			return Ok(Some(vec![Node::String(keyword.to_string(), start_pos)]));
		}}
		self.position = start_pos;
		Ok(None)
	}}

	fn eval(&mut self, expression: &Expression) -> Result<Option<Vec<Node>>, Box<dyn Error>> {{
		match expression {{
			Expression::Rule(rule) => {{
				if let Some(nodes) = self.call_rule(rule, true)? {{
					Ok(Some(nodes))
				}} else {{
					Ok(None)
				}}
			}},
			Expression::RegexLiteral(regex) => self.expect_regex(regex),
			Expression::StringLiteral(string) => self.expect_string(string),
			Expression::Keyword(keyword) => self.expect_keyword(keyword),
			Expression::Or(left, right) => {{
				let start_pos = self.position;
				let left_nodes = self.eval(&*left)?;
				let left_end = self.position;
				self.position = start_pos;
				let right_nodes = self.eval(&*right)?;
				let right_end = self.position;

				if left_end > right_end {{
					self.position = left_end;
					return Ok(left_nodes);
				}} else if right_end > left_end {{
					self.position = right_end;
					return Ok(right_nodes);
				}} else {{
					self.position = start_pos;
					return Ok(None);
				}}
			}},
			Expression::And(left, right) => {{
				let left_nodes = self.eval(&*left)?;
				if left_nodes.is_none() {{
					return Ok(None);
				}}
				let right_nodes = self.eval(&*right)?;
				if right_nodes.is_none() {{
					return Ok(None);
				}}
				let mut final_nodes = left_nodes.unwrap();
				final_nodes.extend(right_nodes.unwrap());
				Ok(Some(final_nodes))
			}},
			Expression::DelimitRepeatOne(expression, delimiter) => {{
				// Attempt to parse the first expression
				let nodes = self.eval(&*expression)?;
				// If the first expression fails, return an empty vector
				if nodes.is_none() {{
					return Ok(None);
				}}

				let mut nodes = nodes.unwrap();

				// Attempt to parse subsequent expressions with delimiters
				loop {{
					// Attempt to parse the delimiter
					let delimiter_nodes = self.eval(&*delimiter)?;
					// If it fails, break the loop
					if delimiter_nodes.is_none() {{
						break;
					}}
					// Attempt to parse the next expression
					let expression_nodes = self.eval(&*expression)?;
					// If the next expression fails, break the loop
					if expression_nodes.is_none() {{
						break;
					}}

					// Only if both delimiter and expression are successful, append them to the nodes
					nodes.extend(delimiter_nodes.unwrap());
					nodes.extend(expression_nodes.unwrap());
				}}

				// Return the nodes collected so far
				Ok(Some(nodes))
			}},
			Expression::DelimitRepeatZero(left, right) => {{
				// Attempt to parse the first expression
				let nodes = self.eval(&*left)?;
				// If the first expression fails, return an empty vector
				if nodes.is_none() {{
					return Ok(Some(vec![]));
				}}

				let mut nodes = nodes.unwrap();

				// Attempt to parse subsequent expressions with delimiters
				loop {{
					// Attempt to parse the delimiter
					let delimiter_nodes = self.eval(&*right)?;
					// If it fails, break the loop
					if delimiter_nodes.is_none() {{
						break;
					}}
					// Attempt to parse the next expression
					let expression_nodes = self.eval(&*left)?;
					// If the next expression fails, break the loop
					if expression_nodes.is_none() {{
						break;
					}}

					// Only if both delimiter and expression are successful, append them to the nodes
					nodes.extend(delimiter_nodes.unwrap());
					nodes.extend(expression_nodes.unwrap());
				}}

				// Return the nodes collected so far
				Ok(Some(nodes))
			}},
			Expression::RepeatOne(expr) => {{
				let mut nodes = self.eval(&*expr)?;

				if nodes.is_none() {{ return Ok(None); }}

				while let Some(new_nodes) = self.eval(&*expr)? {{
					nodes.as_mut().unwrap().extend(new_nodes);
				}}

				Ok(nodes)
			}},
			Expression::RepeatZero(expr) => {{
				let mut nodes = self.eval(&*expr)?;

				if nodes.is_none() {{ return Ok(Some(vec![])); }}

				while let Some(new_nodes) = self.eval(&*expr)? {{
					nodes.as_mut().unwrap().extend(new_nodes);
				}}

				Ok(nodes)
			}},
			Expression::Optional(expr) => {{
				let mut nodes = self.eval(&*expr)?;

				if nodes.is_none() {{
					nodes = Some(vec![]);
				}}

				Ok(nodes)
			}},
		}}
	}}

	fn get_longest_expression_match(&mut self, expressions: &[Expression]) -> Result<Option<Vec<Node>>, Box<dyn Error>> {{
		let start_pos = self.position;
		let mut longest_end = start_pos;
		let mut longest_nodes = Vec::new();

		for expr in expressions.iter() {{
			if let Some(nodes) = self.eval(&expr)? {{
				let new_end_pos = self.position;
				self.position = start_pos; // Reset position to start for each expression evaluation
				if new_end_pos > longest_end || longest_nodes.len() == 0 {{
					longest_end = new_end_pos;
					longest_nodes = nodes;
				}}
			}}
		}}
		if longest_nodes.is_empty() {{
			self.position = start_pos; // Reset position if no matches found
			Ok(None)
		}}else {{
			self.position = longest_end; // Update position to the end of the longest match
			Ok(Some(longest_nodes))
		}}
	}}

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
		format!("_{}", self.gramspec.config.entry_rule),
		self.generate_call_rule(),
		self.generate_rule_functions()?,
		self.generate_meta_rule_functions()?,
		))
	}

	fn generate_call_rule(&self) -> String {
		String::from(format!(

"
	fn call_rule(&mut self, rule_name: &str, protected: bool) -> Result<Option<Vec<Node>>, Box<dyn Error>> {{
		match rule_name {{
{}
			_ => Err(format!(\"Unknown rule: {{}}\", rule_name).into()),
		}}
	}}
",

			{
				let mut cases = String::from("\t\t\t// Regular Rules\n");
				for rule in self.gramspec.rules.keys() {
					if self.gramspec.is_left_circular(rule) {
						cases.push_str(&format!(

"			\"{}\" => {{
				if protected {{
					self.circular_wrapper(\"{}\".to_string())
				}}
				else {{
					self.{}()
				}}
			}}
",
							rule,
							rule,
							format!("_{}", rule)
						));
					} else {
						cases.push_str(&format!(
							"\t\t\t\"{}\" => self.{}(),\n",
							rule,
							format!("_{}", rule)
						));
					}
				}
				cases.push_str("\t\t\t// Meta Rules\n");
				for rule in self.gramspec.meta_rules.keys() {
					if self.gramspec.is_left_circular(rule) {
						cases.push_str(&format!(

"			\"{}\" => {{
				if protected {{
					self.circular_wrapper(\"{}\".to_string())
				}}
				else {{
					self.{}()
				}}
			}}
",
							rule,
							rule,
							format!("_{}", rule)
						));
					} else {
						cases.push_str(&format!(
							"\t\t\t\"{}\" => return self.{}(),\n",
							rule,
							format!("_{}", rule)
						));
					}
				}
				cases
			}

		))
	}

	fn generate_rule_functions(&self) -> Result<String, Box<dyn Error>> {
		let mut functions = String::from("");
		for rule in self.gramspec.rules.keys() {
			let token_expression = self.gramspec.rules.get(rule)
				.or_else(|| self.gramspec.meta_rules.get(rule))
				.ok_or_else(|| format!("Rule '{}' not found", rule))?;

			functions.push_str(format!(

"
	fn {}(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {{
		let start_pos = self.position;
		let expressions: [Expression; {}] = [
{}		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {{
			let mut node = Node::Rule(\"{}\".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}}

		Ok(None)
	}}
",

			// Function Name: Prefix with underscore
			format!("_{}", rule),

			// Number of alternatives; used for array length
			token_expression.len(),

			// Generate the alternative expressions
			{
				let mut alternatives = String::from("");
				for alternative in token_expression {
					alternatives.push_str(&format!("\t\t\t{},\n", self.to_conditional(alternative)?));
				}
				alternatives
			},

			// Rule name for the Node
			rule,

			).as_str());
		}
		Ok(functions)
	}

	fn generate_meta_rule_functions(&self) -> Result<String, Box<dyn Error>> {
		let mut functions = String::from("");
		for rule in self.gramspec.meta_rules.keys() {
			let token_expression = self.gramspec.rules.get(rule)
				.or_else(|| self.gramspec.meta_rules.get(rule))
				.ok_or_else(|| format!("meta-rule '{}' not found", rule))?;

			functions.push_str(&format!(

"
	fn {}(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {{
		let expressions: [Expression; {}] = [
{}		];

		self.get_longest_expression_match(&expressions)
	}}
",

			// Function Name: Prefix with underscore if it's a reserved keyword
			format!("_{}", rule),

			// Number of alternatives; used for array length
			token_expression.len(),

			// Generate the alternative expressions
			{
				let mut alternatives = String::from("");
				for alternative in token_expression {
					alternatives.push_str(&format!("\t\t\t{},\n", self.to_conditional(alternative)?));
				}
				alternatives
			},

			).as_str());
		}
		Ok(functions)
	}

	fn to_conditional(&self, expression: &Expression) -> Result<String, Box<dyn Error>> {
		match expression {
			Expression::RuleName(name) => Ok(format!("Rule(\"{}\")", name.value)),
			Expression::Keyword(keyword) => Ok(format!("Keyword(\"{}\")", keyword.value)),
			Expression::RegexLiteral(regex) => Ok(format!("RegexLiteral(r#\"^{}\"#)", regex.value)),
			Expression::StringLiteral(string) => {
				if string.value == "\"" {
					Ok("StringLiteral(\"\\\"\")".to_string())
				} else if string.value == "\\" {
					Ok("StringLiteral(\"\\\\\")".to_string())
				} else if string.value == "\n" {
					Ok("StringLiteral(\"\\n\")".to_string())
				} else if string.value == "\t" {
					Ok("StringLiteral(\"\\t\")".to_string())
				} else {
					Ok(format!("StringLiteral(\"{}\")", string.value))
				}
			},
			Expression::Or(left, right) => Ok(format!("Expression::or({}, {})", self.to_conditional(left)?, self.to_conditional(right)?)),
			Expression::And(left, right) => Ok(format!("Expression::and({}, {})", self.to_conditional(left)?, self.to_conditional(right)?)),
			Expression::DelimitRepeatOne(left, right) => Ok(format!("Expression::delimit_repeat_one({}, {})", self.to_conditional(left)?, self.to_conditional(right)?)),
			Expression::DelimitRepeatZero(left, right) => Ok(format!("Expression::delimit_repeat_zero({}, {})", self.to_conditional(left)?, self.to_conditional(right)?)),
			Expression::Optional(expr) => Ok(format!("Expression::optional({})", self.to_conditional(expr)?)),
			Expression::RepeatOne(expr) => Ok(format!("Expression::repeat_one({})", self.to_conditional(expr)?)),
			Expression::RepeatZero(expr) => Ok(format!("Expression::repeat_zero({})", self.to_conditional(expr)?)),
		}
	}
}