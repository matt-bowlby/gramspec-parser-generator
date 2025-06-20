use std::error::Error;
use std::collections::HashMap;

use crate::gramspec_parser::gramspec::expression;
use crate::parser::node::Node;
use regex::Regex;

const KEYWORDS: &[(&str, &str)] = &[
	("ENDMARKER", "\0"),
];

pub struct Lang {
	pub position: usize,
	name: &'static str,
	content: String,

	keywords: HashMap<String, String>,
}

impl Lang {

	pub fn new(name: &'static str, content: String) -> Self {
		Lang {
			name,
			position: 0,
			content,
			keywords: HashMap::new(),
		}
	}

	fn get_keywords_map(&self) -> HashMap<String, String> {
		KEYWORDS.iter()
			.map(|(k, v)| (k.to_string(), v.to_string()))
			.collect()
	}

	pub fn expect_string(&mut self, string: &str) -> Result<Vec<Node>, Box<dyn Error>> {

		let start_pos = self.position;
		if self.content[self.position..].starts_with(string) {
			self.position += string.len();
			return Ok(vec![Node::String(string.to_string())]);
		}
		self.position = start_pos;
		Ok(vec![])

	}

	pub fn expect_regex(&mut self, regex: &Regex) -> Result<Vec<Node>, Box<dyn Error>> {
		let start_pos = self.position;
		if let Some(captures) = regex.captures(&self.content[self.position..]) {
			self.position += captures.get(0).unwrap().end();
			return Ok(vec![Node::String(captures.get(0).unwrap().as_str().to_string())]);
		}
		self.position = start_pos;
		Ok(vec![])
	}

	pub fn expect_keyword(&mut self, keyword: &str) -> Result<Vec<Node>, Box<dyn Error>> {
		let start_pos = self.position;
		let keyword_value = self.get_keywords_map().get(keyword)
			.ok_or_else(|| format!("Unknown keyword: {}", keyword))?
			.to_owned();
		if self.content[self.position..].starts_with(&keyword_value) {
			self.position += keyword_value.len();
			return Ok(vec![Node::String(keyword.to_string())]);
		}
		self.position = start_pos;
		Ok(vec![])
	}

	pub fn expect_and<F1, F2>(
		&mut self,
		left: F1,
		right: F2
	) -> Result<Vec<Node>, Box<dyn Error>>
	where
		F1: Fn(&mut Lang) -> Result<Vec<Node>, Box<dyn Error>>,
		F2: Fn(&mut Lang) -> Result<Vec<Node>, Box<dyn Error>>,
	{
		let left_nodes = left(self)?;
		if left_nodes.is_empty() {
			return Ok(vec![]);
		}
		let right_nodes = right(self)?;
		if right_nodes.is_empty() {
			return Ok(vec![]);
		}
		let mut final_nodes = left_nodes;
		final_nodes.extend(right_nodes);
		Ok(final_nodes)
	}

	pub fn expect_or<F1, F2>(
		&mut self,
		left: F1,
		right: F2
	) -> Result<Vec<Node>, Box<dyn Error>>
	where
		F1: Fn(&mut Lang) -> Result<Vec<Node>, Box<dyn Error>>,
		F2: Fn(&mut Lang) -> Result<Vec<Node>, Box<dyn Error>>,
	{
		let start_pos = self.position;
		let left_nodes = left(self)?;
		let left_end = self.position;
		self.position = start_pos;
		let right_nodes = right(self)?;
		let right_end = self.position;
		self.position = start_pos;

		if left_end > right_end {
			self.position = left_end;
			return Ok(left_nodes);
		} else if right_end > left_end {
			self.position = right_end;
			return Ok(right_nodes);
		}
		self.position = start_pos;
		Ok(vec![])
	}

	pub fn expect_delimit_repeat_one<F1, F2>(
		&mut self,
		expression: F1,
		delimiter: F2
	) -> Result<Vec<Node>, Box<dyn Error>>
	where
		F1: Fn(&mut Lang) -> Result<Vec<Node>, Box<dyn Error>>,
		F2: Fn(&mut Lang) -> Result<Vec<Node>, Box<dyn Error>>,
	{
		// Attempt to parse the first expression
		let mut nodes = expression(self)?;
		// If the first expression fails, return an empty vector
		if nodes.is_empty() {
			return Ok(nodes);
		}

		// Attempt to parse subsequent expressions with delimiters
		loop {
			// Attempt to parse the delimiter
			let mut delimiter_nodes = delimiter(self)?;
			// If it fails, break the loop
			if delimiter_nodes.is_empty() {
				break;
			}
			// Attempt to parse the next expression
			let mut expression_nodes = expression(self)?;
			// If the next expression fails, break the loop
			if expression_nodes.is_empty() {
				break;
			}

			// Only if both delimiter and expression are successful, append them to the nodes
			nodes.append(&mut delimiter_nodes);
			nodes.append(&mut expression_nodes);
		}

		// Return the nodes collected so far
		Ok(nodes)
	}

}