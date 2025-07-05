mod expression;
mod node;

use std::error::Error;
use std::collections::HashMap;
use regex::Regex;

use expression::Expression::{self, *};
use node::Node;

const KEYWORDS: &[(&str, &str)] = &[
	("ENDMARKER", "0"),
];

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
		if let Some(nodes) = self._file()? {
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

			let result = self.call_rule(&rule_name, false)?;
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
			if let Some(whitespace) = Regex::new(r"^[ \t]+").unwrap().find(&self.content[self.position..]) {
				self.position += whitespace.end();
			}
			return Ok(Some(vec![Node::String(string.to_string(), start_pos)]));
		}
		self.position = start_pos;
		Ok(None)
	}

	fn expect_regex(&mut self, regex: &str) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		if let Some(captures) = Regex::new(regex).unwrap().captures(&self.content[self.position..]) {
			self.position += captures.get(0).unwrap().end();
			if let Some(whitespace) = Regex::new(r"^[ \t]+").unwrap().find(&self.content[self.position..]) {
				self.position += whitespace.end();
			}
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
				if let Some(nodes) = self.call_rule(rule, true)? {
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

	fn get_longest_expression_match(&mut self, expressions: &[Expression]) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let mut longest_end = start_pos;
		let mut longest_nodes = Vec::new();

		for expr in expressions.iter() {
			if let Some(nodes) = self.eval(&expr)? {
				let new_end_pos = self.position;
				self.position = start_pos; // Reset position to start for each expression evaluation
				if new_end_pos > longest_end || longest_nodes.len() == 0 {
					longest_end = new_end_pos;
					longest_nodes = nodes;
				}
			}
		}
		if longest_nodes.is_empty() {
			self.position = start_pos; // Reset position if no matches found
			Ok(None)
		}else {
			self.position = longest_end; // Update position to the end of the longest match
			Ok(Some(longest_nodes))
		}
	}


	fn call_rule(&mut self, rule_name: &str, protected: bool) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		match rule_name {
			// Regular Rules
			"conversion" => {
				if protected {
					self.circular_wrapper("conversion".to_string())
				}
				else {
					self._conversion()
				}
			}
			"dictionary_lookup" => self._dictionary_lookup(),
			"type" => {
				if protected {
					self.circular_wrapper("type".to_string())
				}
				else {
					self._type()
				}
			}
			"identifier" => self._identifier(),
			"dictionary_type" => {
				if protected {
					self.circular_wrapper("dictionary_type".to_string())
				}
				else {
					self._dictionary_type()
				}
			}
			"equation" => {
				if protected {
					self.circular_wrapper("equation".to_string())
				}
				else {
					self._equation()
				}
			}
			"boolean_value" => self._boolean_value(),
			"list_type" => {
				if protected {
					self.circular_wrapper("list_type".to_string())
				}
				else {
					self._list_type()
				}
			}
			"indent" => self._indent(),
			"integer_value" => self._integer_value(),
			"variable_update" => self._variable_update(),
			"list_lookup" => self._list_lookup(),
			"boolean_expression" => {
				if protected {
					self.circular_wrapper("boolean_expression".to_string())
				}
				else {
					self._boolean_expression()
				}
			}
			"function_definition" => self._function_definition(),
			"repeat_statement" => self._repeat_statement(),
			"if_statement" => self._if_statement(),
			"float_value" => self._float_value(),
			"file" => self._file(),
			"while_statement" => self._while_statement(),
			"function_call" => self._function_call(),
			"comparison" => {
				if protected {
					self.circular_wrapper("comparison".to_string())
				}
				else {
					self._comparison()
				}
			}
			"comment" => self._comment(),
			"variable_definition" => self._variable_definition(),
			"string_value" => self._string_value(),
			"object_create" => self._object_create(),
			// Meta Rules
			"block_line" => return self._block_line(),
			"value" => {
				if protected {
					self.circular_wrapper("value".to_string())
				}
				else {
					self._value()
				}
			}
			"control_statement" => return self._control_statement(),
			"lines" => return self._lines(),
			"boolean_operator" => return self._boolean_operator(),
			"function_keyword" => return self._function_keyword(),
			"math_operator" => return self._math_operator(),
			"line" => return self._line(),
			"member" => return self._member(),
			"comparator_operator" => return self._comparator_operator(),
			"variable_keyword" => return self._variable_keyword(),

			_ => Err(format!("Unknown rule: {}", rule_name).into()),
		}
	}


	fn _conversion(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Rule("value"), StringLiteral(r#"as"#)), Rule("type")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("conversion".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _dictionary_lookup(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(StringLiteral(r#"lookup"#), Rule("value")), StringLiteral(r#"in"#)), Rule("identifier")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("dictionary_lookup".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _type(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 9] = [
			StringLiteral(r#"integer"#),
			StringLiteral(r#"float"#),
			StringLiteral(r#"string"#),
			StringLiteral(r#"boolean"#),
			StringLiteral(r#"general"#),
			StringLiteral(r#"nothing"#),
			Rule("list_type"),
			Rule("dictionary_type"),
			Rule("identifier"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("type".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _identifier(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			RegexLiteral(r#"[a-z][a-z0-9_]*"#),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("identifier".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _dictionary_type(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(Expression::and(Rule("type"), StringLiteral(r#"-"#)), Rule("type")), StringLiteral(r#"-"#)), StringLiteral(r#"dictionary"#)),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("dictionary_type".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _equation(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 6] = [
			Expression::and(Expression::and(Rule("equation"), Rule("math_operator")), Rule("equation")),
			Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral(r#"("#), Rule("equation")), StringLiteral(r#")"#)), Rule("math_operator")), Rule("equation")),
			Expression::and(Expression::and(StringLiteral(r#"("#), Rule("equation")), StringLiteral(r#")"#)),
			Rule("value"),
			Rule("dictionary_lookup"),
			Rule("list_lookup"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("equation".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _boolean_value(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 2] = [
			StringLiteral(r#"true"#),
			StringLiteral(r#"false"#),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("boolean_value".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _list_type(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Rule("type"), StringLiteral(r#"-"#)), StringLiteral(r#"list"#)),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("list_type".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _indent(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			RegexLiteral(r#"[\t ]*"#),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("indent".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _integer_value(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			RegexLiteral(r#"[+-]?[0-9]+"#),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("integer_value".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _variable_update(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(StringLiteral(r#"change"#), Rule("identifier")), Expression::or(StringLiteral(r#"to"#), StringLiteral(r#"by"#))), Expression::or(Expression::or(Rule("value"), Rule("function_call")), Rule("equation"))),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("variable_update".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _list_lookup(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(StringLiteral(r#"lookup"#), Rule("integer_value")), StringLiteral(r#"in"#)), Rule("identifier")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("list_lookup".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _boolean_expression(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 6] = [
			Expression::and(Expression::and(Rule("boolean_expression"), Rule("boolean_operator")), Rule("boolean_expression")),
			Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral(r#"("#), Rule("boolean_expression")), StringLiteral(r#")"#)), Rule("boolean_operator")), Rule("boolean_expression")),
			Expression::and(Expression::and(StringLiteral(r#"("#), Rule("boolean_expression")), StringLiteral(r#")"#)),
			Rule("comparison"),
			StringLiteral(r#"true"#),
			StringLiteral(r#"false"#),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("boolean_expression".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _function_definition(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral(r#"create"#), Expression::repeat_zero(Rule("function_keyword"))), Rule("type")), StringLiteral(r#"function"#)), Rule("identifier")), StringLiteral(r#":"#)), StringLiteral(r#"\n"#)), Expression::repeat_one(Rule("block_line"))),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("function_definition".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _repeat_statement(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral(r#"repeat"#), Rule("integer_value")), StringLiteral(r#"times"#)), StringLiteral(r#":"#)), StringLiteral(r#"\n"#)), Expression::repeat_one(Rule("block_line"))),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("repeat_statement".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _if_statement(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral(r#"if"#), Rule("boolean_expression")), StringLiteral(r#":"#)), StringLiteral(r#"\n"#)), Expression::repeat_one(Rule("block_line"))),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("if_statement".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _float_value(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			RegexLiteral(r#"[+-]?[0-9]+\.[0-9]+"#),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("float_value".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _file(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Rule("lines"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("file".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _while_statement(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral(r#"while"#), Rule("boolean_expression")), StringLiteral(r#":"#)), StringLiteral(r#"\n"#)), Expression::repeat_one(Rule("block_line"))),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("while_statement".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _function_call(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(StringLiteral(r#"call"#), Rule("identifier")), Expression::optional(Expression::and(StringLiteral(r#"with"#), Expression::or(Rule("equation"), Expression::and(Expression::and(Expression::delimit_repeat_one(Rule("equation"), StringLiteral(r#","#)), StringLiteral(r#"and"#)), Rule("equation")))))),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("function_call".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _comparison(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 5] = [
			Expression::and(Expression::and(Rule("comparison"), Rule("comparator_operator")), Rule("comparison")),
			Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral(r#"("#), Rule("comparison")), StringLiteral(r#")"#)), Rule("comparator_operator")), Rule("comparison")),
			Expression::and(Expression::and(StringLiteral(r#"("#), Rule("comparison")), StringLiteral(r#")"#)),
			Rule("equation"),
			StringLiteral(r#"nothing"#),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("comparison".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _comment(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(StringLiteral(r#"["#), RegexLiteral(r#"(?:[^\\[\]]|\\.)*"#)), StringLiteral(r#"]"#)),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("comment".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _variable_definition(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral(r#"create"#), Expression::repeat_zero(Rule("variable_keyword"))), Rule("type")), StringLiteral(r#"variable"#)), Rule("identifier")), StringLiteral(r#"with"#)), Expression::or(Expression::or(Rule("value"), Rule("dictionary_lookup")), Rule("list_lookup"))),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("variable_definition".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _string_value(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(StringLiteral(r#"""#), RegexLiteral(r#"([^"\\]|\\.)*"#)), StringLiteral(r#"""#)),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("string_value".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _object_create(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(StringLiteral(r#"new"#), Rule("identifier")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("object_create".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}


	fn _block_line(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Rule("indent"), Expression::or(Expression::or(Expression::or(Expression::or(Expression::or(Expression::or(Rule("comment"), Rule("variable_definition")), Rule("variable_update")), Rule("function_call")), Rule("control_statement")), StringLiteral(r#"return"#)), StringLiteral(r#"pass"#))), StringLiteral(r#"\n"#)),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _value(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 8] = [
			Rule("integer_value"),
			Rule("float_value"),
			Rule("boolean_value"),
			Rule("string_value"),
			Rule("object_create"),
			Rule("conversion"),
			Rule("identifier"),
			StringLiteral(r#"nothing"#),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _control_statement(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 3] = [
			Rule("if_statement"),
			Rule("while_statement"),
			Rule("repeat_statement"),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _lines(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 1] = [
			Expression::repeat_one(Rule("line")),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _boolean_operator(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 3] = [
			StringLiteral(r#"and"#),
			StringLiteral(r#"or"#),
			StringLiteral(r#"not"#),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _function_keyword(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 2] = [
			StringLiteral(r#"public"#),
			StringLiteral(r#"private"#),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _math_operator(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 6] = [
			StringLiteral(r#"+"#),
			StringLiteral(r#"-"#),
			StringLiteral(r#"*"#),
			StringLiteral(r#"/"#),
			StringLiteral(r#"%"#),
			StringLiteral(r#"^"#),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _line(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(Rule("indent"), Rule("member")), StringLiteral(r#"."#)), Expression::repeat_zero(StringLiteral(r#"\n"#))),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _member(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 3] = [
			Rule("comment"),
			Rule("variable_definition"),
			Rule("function_definition"),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _comparator_operator(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 10] = [
			StringLiteral(r#"=="#),
			StringLiteral(r#"is"#),
			StringLiteral(r#"!="#),
			StringLiteral(r#"<"#),
			StringLiteral(r#"<="#),
			StringLiteral(r#">"#),
			StringLiteral(r#">="#),
			StringLiteral(r#"and"#),
			StringLiteral(r#"or"#),
			StringLiteral(r#"not"#),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _variable_keyword(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 5] = [
			StringLiteral(r#"public"#),
			StringLiteral(r#"private"#),
			StringLiteral(r#"input"#),
			StringLiteral(r#"output"#),
			StringLiteral(r#"index"#),
		];

		self.get_longest_expression_match(&expressions)
	}

}