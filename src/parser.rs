use std::error::Error;
use std::collections::HashMap;


use crate::parser::node::Node;
use crate::parser::lang::Lang;
use crate::parser::expression::Expression::*;

pub mod node;

mod expression;
mod lang;

#[allow(dead_code)]
/// Generated Grammar Specification
pub struct Parser {
	memos: HashMap<String, HashMap<String, Option<String>>>,
}

#[allow(dead_code)]
impl Parser {

	pub fn new() -> Self {
		Parser { memos: HashMap::new() }
	}

	pub fn parse(&mut self, input: String) -> Result<Option<Node>, Box<dyn Error>> {
		let mut lang = Lang::new("TestLanguage", input);
		if let Some(nodes) = self.file(&mut lang)? {
			return Ok(Some(nodes[0].clone()));
		}
		Ok(None)
	}

	pub fn parse_file(&mut self, file_path: &str) -> Result<Option<Node>, Box<dyn Error>> {
		let content = std::fs::read_to_string(file_path)?;
		self.parse(content)
	}

	pub(crate) fn call_rule(&self, rule_name: &str, lang: &mut Lang) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		match rule_name {
			// Regular Rules
			"hello" =>  return self.hello(lang),
			"file" =>  return self.file(lang),

			// Meta Rules
			"test" =>  return self.test(lang),
			_ => Err(format!("Unknown rule: {}", rule_name).into()),
		}
	}

	pub(crate) fn hello(&self, lang: &mut Lang) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let mut node = Node::Rule("hello".to_string(), Vec::new());

		let start_pos = lang.position;
		if let Some(nodes) = StringLiteral("yaga").eval(lang, self)? {
			node.extend(nodes);
			return Ok(Some(vec![node]));
		}
		lang.position = start_pos;

		Ok(None)
	}

	pub(crate) fn file(&self, lang: &mut Lang) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		// Left recursive
		let mut node = Node::Rule("file".to_string(), Vec::new());

		let start_pos = lang.position;
		if let Some(nodes) = DelimitRepeatZero(Box::new(Rule("test")), Box::new(StringLiteral("hello"))).eval(lang, self)? {
			node.extend(nodes);
			return Ok(Some(vec![node]));
		}
		lang.position = start_pos;

		if let Some(nodes) = Rule("hello").eval(lang, self)? {
			node.extend(nodes);
			return Ok(Some(vec![node]));
		}
		lang.position = start_pos;

		if let Some(nodes) = StringLiteral("bruh").eval(lang, self)? {
			node.extend(nodes);
			return Ok(Some(vec![node]));
		}
		lang.position = start_pos;

		Ok(None)
	}

	fn test(&self, lang: &mut Lang) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		// Left recursive
		let mut nodes = Vec::new();

		let start_pos = lang.position;
		if let Some(result_nodes) = And(Box::new(Rule("file")), Box::new(RegexLiteral(r"\d+"))).eval(lang, self)? {
			nodes.extend(result_nodes);
			return Ok(Some(nodes));
		} else {
			lang.position = start_pos;
		}
		Ok(None)
	}

}