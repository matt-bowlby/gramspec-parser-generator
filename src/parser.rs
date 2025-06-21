use std::error::Error;
use crate::parser::node::Node;
use crate::parser::lang::Lang;
use crate::parser::expression::Expression::*;

pub mod node;

mod expression;
mod lang;

#[allow(dead_code)]
/// Generated Grammar Specification
struct TestLanguage { }

#[allow(dead_code)]
impl TestLanguage {

	pub fn new() -> Self {
		TestLanguage { }
	}

	pub fn parse(&mut self, input: String) -> Result<Option<Node>, Box<dyn Error>> {
		let mut lang = Lang::new("TestLanguage", input);
		self.file(&mut lang)?;
		Ok(None)
	}

	fn test(&mut self, lang: &mut Lang) -> Result<Option<Node>, Box<dyn Error>> {
		let mut node = Node::Rule("test".to_string(), Vec::new());

		let start_pos = lang.position;
		if let Some(nodes) = RegexLiteral(r"\d+").eval(lang)? {
			node.extend(nodes);
			return Ok(Some(node));
		}
		lang.position = start_pos;

		Ok(None)
	}

	fn hello(&mut self, lang: &mut Lang) -> Result<Option<Node>, Box<dyn Error>> {
		let mut node = Node::Rule("hello".to_string(), Vec::new());

		let start_pos = lang.position;
		if let Some(nodes) = StringLiteral("yaga").eval(lang)? {
			node.extend(nodes);
			return Ok(Some(node));
		}
		lang.position = start_pos;

		Ok(None)
	}

	fn file(&mut self, lang: &mut Lang) -> Result<Option<Node>, Box<dyn Error>> {
		let mut node = Node::Rule("file".to_string(), Vec::new());

		let start_pos = lang.position;
		if let Some(nodes) = DelimitRepeatOne(Box::new(RuleName("test")), Box::new(And(Box::new(StringLiteral("hello")), Box::new(RuleName("test"))))).eval(lang)? {
			node.extend(nodes);
			return Ok(Some(node));
		}
		lang.position = start_pos;

		if let Some(nodes) = RuleName("hello").eval(lang)? {
			node.extend(nodes);
			return Ok(Some(node));
		}
		lang.position = start_pos;

		if let Some(nodes) = StringLiteral("bruh").eval(lang)? {
			node.extend(nodes);
			return Ok(Some(node));
		}
		lang.position = start_pos;

		Ok(None)
	}

}