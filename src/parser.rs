use std::error::Error;

use crate::parser::node::Node;
use crate::parser::lang::Lang;

pub mod node;
mod lang;

/// Generated Grammar Specification
struct TestLanguage { }

impl TestLanguage {

	pub fn new() -> Self {
		TestLanguage { }
	}

	pub fn parse(&mut self, input: String) -> Result<Option<Node>, Box<dyn Error>> {
		let mut lang = Lang::new("TestLanguage", input);
		self.file(&mut lang)?;
		Ok(None)
	}

	fn file(&mut self, lang: &mut Lang) -> Result<Option<Node>, Box<dyn Error>> {
		let mut node = Node::Rule("file".to_string(), Vec::new());

		let start_pos = lang.position;
		if let Some(result_node) = lang.expect_delimit_repeat_one(self.test(lang, node)?, lang.expect_and(lang.expect_string("hello", node)?, self.test(lang, node)?, node)?, node)? {
			return Ok(Some(result_node));
		}
		lang.position = start_pos;

		if let Some(result_node) = self.hello(lang, node)? {
			return Ok(Some(result_node));
		}
		lang.position = start_pos;

		if let Some(result_node) = lang.expect_string("bruh", node)? {
			return Ok(Some(result_node));
		}
		lang.position = start_pos;

		Ok(None)
	}

	fn test(&mut self, lang: &mut Lang) -> Result<Option<Node>, Box<dyn Error>> {
		let mut node = Node::Rule("test".to_string(), Vec::new());

		let start_pos = lang.position;
		if let Some(result_node) = lang.expect_regex("\d+", node)? {
			return Ok(Some(result_node));
		}
		lang.position = start_pos;

		Ok(None)
	}

	fn hello(&mut self, lang: &mut Lang) -> Result<Option<Node>, Box<dyn Error>> {
		let mut node = Node::Rule("hello".to_string(), Vec::new());

		let start_pos = lang.position;
		if let Some(result_node) = lang.expect_string("yaga", node)? {
			return Ok(Some(result_node));
		}
		lang.position = start_pos;

		Ok(None)
	}

}