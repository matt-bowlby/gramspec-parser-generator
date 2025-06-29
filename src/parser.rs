use std::error::Error;
use std::collections::HashMap;
use regex::Regex;
use std::fmt;

use crate::parser::Expression::*;

const KEYWORDS: &[(&str, &str)] = &[
	("ENDMARKER", "0"),
];

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Node {
	Rule(String, Vec<Box<Node>>, usize),
	String(String, usize),
}

#[allow(dead_code)]
impl Node {
	pub fn append(&mut self, child: Node) {
		match self {
			Node::Rule(_, children, _) => children.push(Box::new(child)),
			_ => return,
		}
	}

	pub fn extend(&mut self, children: Vec<Node>) {
		// Only extend if the current node can be extended
		if !matches!(self, Node::Rule(_, _, _)) { return; }

		// Extend the current node with the provided children
		for child in children {
			self.append(child);
		}
	}

	pub fn get_end_pos(&self) -> usize {
		match self {
			Node::Rule(_, nodes, start_pos) => {
				for node in nodes {
					let end_pos = node.get_end_pos();
					if end_pos > *start_pos {
						return end_pos;
					}
				}
				*start_pos
			},
			Node::String(string, start_pos) => *start_pos + string.len(),
		}
	}
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum Expression {
	Rule(&'static str),
	RegexLiteral(&'static str),
	StringLiteral(&'static str),
	Keyword(&'static str),
	Or(Box<Expression>, Box<Expression>),
	And(Box<Expression>, Box<Expression>),
	DelimitRepeatOne(Box<Expression>, Box<Expression>),
	DelimitRepeatZero(Box<Expression>, Box<Expression>),
	Optional(Box<Expression>),
	RepeatOne(Box<Expression>),
	RepeatZero(Box<Expression>),
}

impl fmt::Debug for Expression {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Expression::Rule(rule) => write!(f, "{}", rule),
			Expression::RegexLiteral(regex) => write!(f, "{}", regex),
			Expression::StringLiteral(string) => write!(f, "{}", string),
			Expression::Keyword(keyword) => write!(f, "{}", keyword),
			Expression::Or(left, right) => write!(f, "{:?} | {:?}", left, right),
			Expression::And(left, right) => write!(f, "{:?} & {:?}", left, right),
			Expression::DelimitRepeatOne(left, right) => write!(f, "({:?}),({:?})+", left, right),
			Expression::DelimitRepeatZero(left, right) => write!(f, "({:?}),({:?})*", left, right),
			Expression::Optional(expr) => write!(f, "({:?})?", expr),
			Expression::RepeatOne(expr) => write!(f, "({:?})+", expr),
			Expression::RepeatZero(expr) => write!(f, "({:?})*", expr),
		}
	}
}

#[allow(dead_code)]
/// Generated Grammar Specification
pub struct Parser {
	pub position: usize,

	content: String,
	memos: HashMap<usize, HashMap<String, Box<Option<Vec<Node>>>>>,
}

#[allow(dead_code)]
impl Parser {
	pub fn new() -> Self {
		Parser { content: String::new(), position: 0, memos: HashMap::new() }
	}

	pub fn parse(&mut self, input: String) -> Result<Option<Node>, Box<dyn Error>> {
		self.position = 0;
		self.content = input;
		if let Some(nodes) = self.file()? {
			return Ok(Some(nodes[0].clone()));
		}
		Ok(None)
	}

	pub fn parse_file(&mut self, file_path: &str) -> Result<Option<Node>, Box<dyn Error>> {
		let content = std::fs::read_to_string(file_path)?;
		self.parse(content)
	}

	fn circular_wrapper(&mut self, rule_name: String) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let pos = self.position;

		if let Some(cached_result) = self.memos.get(&pos).and_then(|memo| memo.get(&rule_name)) {
			self.position = pos;
			return Ok(*cached_result.clone());
		}

		self.memos.entry(pos).or_insert_with(HashMap::new).insert(rule_name.clone(), Box::new(None));

		let mut last_result = None;
		let mut last_pos = pos;

		loop {
			self.position = pos;

			let result = self.call_rule(&rule_name)?;
			let end_pos = self.position;

			if end_pos <= last_pos {
				break;
			}

			last_result = result;
			last_pos = end_pos;

			if let Some(memo) = self.memos.get_mut(&pos) {
				memo.insert(rule_name.clone(), Box::new(last_result.clone()));
			}
		}

		self.position = last_pos;
		Ok(last_result)
	}

	fn expect_string(&mut self, string: &str) -> Result<Option<Vec<Node>>, Box<dyn Error>> {

		let start_pos = self.position;
		if self.content[self.position..].starts_with(string) {
			self.position += string.len();
			return Ok(Some(vec![Node::String(string.to_string(), start_pos)]));
		}
		self.position = start_pos;
		Ok(None)

	}

	fn expect_regex(&mut self, regex: &str) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		if let Some(captures) = Regex::new(regex).unwrap().captures(&self.content[self.position..]) {
			self.position += captures.get(0).unwrap().end();
			return Ok(Some(vec![Node::String(captures.get(0).unwrap().as_str().to_string(), start_pos)]));
		}
		self.position = start_pos;
		Ok(None)
	}

	fn get_keywords_map(&self) -> HashMap<String, String> {
		KEYWORDS.iter()
			.map(|(k, v)| (k.to_string(), v.to_string()))
			.collect()
	}

	fn expect_keyword(&mut self, keyword: &str) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let keyword_value = self.get_keywords_map().get(keyword)
			.ok_or_else(|| format!("Unknown keyword: {}", keyword))?
			.to_owned();
		if self.content[self.position..].starts_with(&keyword_value) {
			self.position += keyword_value.len();
			return Ok(Some(vec![Node::String(keyword.to_string(), start_pos)]));
		}
		self.position = start_pos;
		Ok(None)
	}

	fn eval(&mut self, expression: &Expression) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		match expression {
			Expression::Rule(rule) => {
				if let Some(nodes) = self.call_rule(rule)? {
					Ok(Some(nodes))
				} else {
					Ok(None)
				}
			},
			Expression::RegexLiteral(regex) => self.expect_regex(regex),
			Expression::StringLiteral(string) => self.expect_string(string),
			Expression::Keyword(keyword) => self.expect_keyword(keyword),
			Expression::Or(left, right) => {
				let start_pos = self.position;
				let left_nodes = self.eval(&*left)?;
				let left_end = self.position;
				self.position = start_pos;
				let right_nodes = self.eval(&*right)?;
				let right_end = self.position;

				if left_end > right_end {
					self.position = left_end;
					return Ok(left_nodes);
				} else if right_end > left_end {
					self.position = right_end;
					return Ok(right_nodes);
				} else {
					self.position = start_pos;
					return Ok(None);
				}
			},
			Expression::And(left, right) => {
				let left_nodes = self.eval(&*left)?;
				if left_nodes.is_none() {
					return Ok(None);
				}
				let right_nodes = self.eval(&*right)?;
				if right_nodes.is_none() {
					return Ok(None);
				}
				let mut final_nodes = left_nodes.unwrap();
				final_nodes.extend(right_nodes.unwrap());
				Ok(Some(final_nodes))
			},
			Expression::DelimitRepeatOne(expression, delimiter) => {
				// Attempt to parse the first expression
				let nodes = self.eval(&*expression)?;
				// If the first expression fails, return an empty vector
				if nodes.is_none() {
					return Ok(None);
				}

				let mut nodes = nodes.unwrap();

				// Attempt to parse subsequent expressions with delimiters
				loop {
					// Attempt to parse the delimiter
					let delimiter_nodes = self.eval(&*delimiter)?;
					// If it fails, break the loop
					if delimiter_nodes.is_none() {
						break;
					}
					// Attempt to parse the next expression
					let expression_nodes = self.eval(&*expression)?;
					// If the next expression fails, break the loop
					if expression_nodes.is_none() {
						break;
					}

					// Only if both delimiter and expression are successful, append them to the nodes
					nodes.extend(delimiter_nodes.unwrap());
					nodes.extend(expression_nodes.unwrap());
				}

				// Return the nodes collected so far
				Ok(Some(nodes))
			},
			Expression::DelimitRepeatZero(left, right) => {
				// Attempt to parse the first expression
				let nodes = self.eval(&*left)?;
				// If the first expression fails, return an empty vector
				if nodes.is_none() {
					return Ok(Some(vec![]));
				}

				let mut nodes = nodes.unwrap();

				// Attempt to parse subsequent expressions with delimiters
				loop {
					// Attempt to parse the delimiter
					let delimiter_nodes = self.eval(&*right)?;
					// If it fails, break the loop
					if delimiter_nodes.is_none() {
						break;
					}
					// Attempt to parse the next expression
					let expression_nodes = self.eval(&*left)?;
					// If the next expression fails, break the loop
					if expression_nodes.is_none() {
						break;
					}

					// Only if both delimiter and expression are successful, append them to the nodes
					nodes.extend(delimiter_nodes.unwrap());
					nodes.extend(expression_nodes.unwrap());
				}

				// Return the nodes collected so far
				Ok(Some(nodes))
			},
			Expression::RepeatOne(expr) => {
				let mut nodes = self.eval(&*expr)?;

				if nodes.is_none() { return Ok(None); }

				while let Some(new_nodes) = self.eval(&*expr)? {
					nodes.as_mut().unwrap().extend(new_nodes);
				}

				Ok(nodes)
			},
			Expression::RepeatZero(expr) => {
				let mut nodes = self.eval(&*expr)?;

				if nodes.is_none() { return Ok(Some(vec![])); }

				while let Some(new_nodes) = self.eval(&*expr)? {
					nodes.as_mut().unwrap().extend(new_nodes);
				}

				Ok(nodes)
			},
			Expression::Optional(expr) => {
				let mut nodes = self.eval(&*expr)?;

				if nodes.is_none() {
					nodes = Some(vec![]);
				}

				Ok(nodes)
			},
		}
	}

	fn call_rule(&mut self, rule_name: &str) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		match rule_name {
			// Regular Rules
			"file" =>  return self.file(),
			"hello" =>  return self.hello(),
			// Meta Rules
			"test" =>  return self.test(),

			_ => Err(format!("Unknown rule: {}", rule_name).into()),
		}
	}


	fn file(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let mut node = Node::Rule("file".to_string(), Vec::new(), start_pos);

		// Left recursive

		if let Some(nodes) = self.eval(&DelimitRepeatZero(Box::new(Rule("test")), Box::new(StringLiteral("hello"))))? {
			node.extend(nodes);
			return Ok(Some(vec![node]));
		}
		self.position = start_pos;

		if let Some(nodes) = self.eval(&Rule("hello"))? {
			node.extend(nodes);
			return Ok(Some(vec![node]));
		}
		self.position = start_pos;

		if let Some(nodes) = self.eval(&StringLiteral("bruh"))? {
			node.extend(nodes);
			return Ok(Some(vec![node]));
		}
		self.position = start_pos;

		Ok(None)
	}

	fn hello(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let mut node = Node::Rule("hello".to_string(), Vec::new(), start_pos);


		if let Some(nodes) = self.eval(&StringLiteral("yaga"))? {
			node.extend(nodes);
			return Ok(Some(vec![node]));
		}
		self.position = start_pos;

		Ok(None)
	}


	fn test(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		// Left recursive

		let mut nodes = Vec::new();

		if let Some(result_nodes) = self.eval(&And(Box::new(Rule("file")), Box::new(RegexLiteral(r"\d+"))))? {
			nodes.extend(result_nodes);
			return Ok(Some(nodes));
		} else {
			self.position = start_pos;
		}

		Ok(None)
	}


}