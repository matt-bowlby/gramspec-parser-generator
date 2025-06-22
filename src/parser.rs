use std::error::Error;
use crate::parser::node::Node;
use crate::parser::lang::Lang;
use crate::parser::expression::Expression::*;

pub mod node;

mod expression;
mod lang;

#[allow(dead_code)]
/// Generated Grammar Specification
pub struct Parser { }

#[allow(dead_code)]
impl Parser {

	pub fn new() -> Self {
		Parser { }
	}

	pub fn parse(&mut self, input: String) -> Result<Option<Node>, Box<dyn Error>> {
		let mut lang = Lang::new("TestLanguage", input);
		self.file(&mut lang)
	}

	pub fn parse_file(&mut self, file_path: &str) -> Result<Option<Node>, Box<dyn Error>> {
		let content = std::fs::read_to_string(file_path)?;
		self.parse(content)
	}

	pub(crate) fn call_rule(&self, rule_name: &str, lang: &mut Lang) -> Result<Option<Node>, Box<dyn Error>> {
		match rule_name {
			"file" =>  return self.file(lang),
			"hello" =>  return self.hello(lang),
			"test" =>  return self.test(lang),
			_ => Err(format!("Unknown rule: {}", rule_name).into()),
		}
	}

	pub(crate) fn file(&self, lang: &mut Lang) -> Result<Option<Node>, Box<dyn Error>> {
		let mut node = Node::Rule("file".to_string(), Vec::new());

		let start_pos = lang.position;
		if let Some(nodes) = DelimitRepeatZero(Box::new(Rule("test")), Box::new(StringLiteral("hello"))).eval(lang, self)? {
			node.extend(nodes);
			return Ok(Some(node));
		}
		lang.position = start_pos;

		if let Some(nodes) = Rule("hello").eval(lang, self)? {
			node.extend(nodes);
			return Ok(Some(node));
		}
		lang.position = start_pos;

		if let Some(nodes) = StringLiteral("bruh").eval(lang, self)? {
			node.extend(nodes);
			return Ok(Some(node));
		}
		lang.position = start_pos;

		Ok(None)
	}

	pub(crate) fn hello(&self, lang: &mut Lang) -> Result<Option<Node>, Box<dyn Error>> {
		let mut node = Node::Rule("hello".to_string(), Vec::new());

		let start_pos = lang.position;
		if let Some(nodes) = StringLiteral("yaga").eval(lang, self)? {
			node.extend(nodes);
			return Ok(Some(node));
		}
		lang.position = start_pos;

		Ok(None)
	}

	pub(crate) fn test(&self, lang: &mut Lang) -> Result<Option<Node>, Box<dyn Error>> {
		let mut node = Node::Rule("test".to_string(), Vec::new());

		let start_pos = lang.position;
		if let Some(nodes) = RegexLiteral(r"\d+").eval(lang, self)? {
			node.extend(nodes);
			return Ok(Some(node));
		}
		lang.position = start_pos;

		Ok(None)
	}

}