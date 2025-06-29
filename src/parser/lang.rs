use std::error::Error;
use std::collections::HashMap;
use regex::Regex;

use crate::parser::node::Node;

const KEYWORDS: &[(&str, &str)] = &[
	("ENDMARKER", "\0"),
];

#[allow(dead_code)]
pub struct Lang {
	pub position: usize,

	name: &'static str,
	content: String,
}

#[allow(dead_code)]
impl Lang {

	pub fn new(name: &'static str, content: String) -> Self {
		Lang {
			name,
			position: 0,
			content
		}
	}

	fn get_keywords_map(&self) -> HashMap<String, String> {
		KEYWORDS.iter()
			.map(|(k, v)| (k.to_string(), v.to_string()))
			.collect()
	}

	pub fn expect_string(&mut self, string: &str) -> Result<Option<Vec<Node>>, Box<dyn Error>> {

		let start_pos = self.position;
		if self.content[self.position..].starts_with(string) {
			self.position += string.len();
			return Ok(Some(vec![Node::String(string.to_string(), start_pos)]));
		}
		self.position = start_pos;
		Ok(None)

	}

	pub fn expect_regex(&mut self, regex: &str) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		if let Some(captures) = Regex::new(regex).unwrap().captures(&self.content[self.position..]) {
			self.position += captures.get(0).unwrap().end();
			return Ok(Some(vec![Node::String(captures.get(0).unwrap().as_str().to_string(), start_pos)]));
		}
		self.position = start_pos;
		Ok(None)
	}

	pub fn expect_keyword(&mut self, keyword: &str) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
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

}