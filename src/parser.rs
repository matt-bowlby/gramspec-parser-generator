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
	memos: HashMap<usize, HashMap<String, Box<Option<Vec<Node>>>>>,
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

	fn circular_wrapper(&mut self, lang: &mut Lang, rule_name: String) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
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


	pub(crate) fn call_rule(&self, rule_name: &str, lang: &mut Lang) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		match rule_name {
			// Regular Rules
			"file" =>  return self.file(lang),
			"hello" =>  return self.hello(lang),
			// Meta Rules
			"test" =>  return self.test(lang),

			_ => Err(format!("Unknown rule: {}", rule_name).into()),
		}
	}


	pub(crate) fn file(&self, lang: &mut Lang) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = lang.position;
		let mut node = Node::Rule("file".to_string(), Vec::new(), start_pos);

		// Left recursive

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

	pub(crate) fn hello(&self, lang: &mut Lang) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = lang.position;
		let mut node = Node::Rule("hello".to_string(), Vec::new(), start_pos);


		if let Some(nodes) = StringLiteral("yaga").eval(lang, self)? {
			node.extend(nodes);
			return Ok(Some(vec![node]));
		}
		lang.position = start_pos;

		Ok(None)
	}


	fn test(&self, lang: &mut Lang) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = lang.position;
		// Left recursive

		let mut nodes = Vec::new();

		if let Some(result_nodes) = And(Box::new(Rule("file")), Box::new(RegexLiteral(r"\d+"))).eval(lang, self)? {
			nodes.extend(result_nodes);
			return Ok(Some(nodes));
		} else {
			lang.position = start_pos;
		}

		Ok(None)
	}


}