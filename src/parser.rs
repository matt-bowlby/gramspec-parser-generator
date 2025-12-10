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

		if let Some(cached_result_box) = self.memos.get(&pos).and_then(|memo| memo.get(&rule_name)) {
			let cached_result = *cached_result_box.clone();

			let end_pos = cached_result.as_ref()
				.and_then(|nodes| nodes.iter().map(|n| n.get_end_pos()).max())
				.unwrap_or(pos);
			self.position = end_pos;

			return Ok(cached_result);
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
		let mut start_pos = self.position;
		loop {
			if self.content[self.position..].starts_with(string) {
				self.position += string.len();
				return Ok(Some(vec![Node::String(string.to_string(), start_pos)]));
			} else {
				if let Some(whitespace) = Regex::new(r"^\s+").unwrap().find(&self.content[self.position..]) {
					start_pos += whitespace.end();
					self.position = start_pos;
				} else {
					break;
				}
			}
		}
		self.position = start_pos;
		Ok(None)
	}

	fn expect_regex(&mut self, regex: &str) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let mut start_pos = self.position;
		loop {
			if let Some(captures) = Regex::new(regex).unwrap().captures(&self.content[self.position..]) {
				self.position += captures.get(0).unwrap().end();
				return Ok(Some(vec![Node::String(captures.get(0).unwrap().as_str().to_string(), start_pos)]));
			} else {
				if let Some(whitespace) = Regex::new(r"^\s+").unwrap().find(&self.content[self.position..]) {
					start_pos += whitespace.end();
					self.position = start_pos;
				} else {
					break;
				}
			}
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
					let start = self.position;
					// Attempt to parse the delimiter
					let delimiter_nodes = self.eval(&*delimiter)?;
					// If it fails, break the loop
					if delimiter_nodes.is_none() {
						self.position = start; // Technically unnecessary as a failure would leave position unchanged, but just to be consistent
						break;
					}
					// Attempt to parse the next expression
					let expression_nodes = self.eval(&*expression)?;
					// If the next expression fails, break the loop
					if expression_nodes.is_none() {
						self.position = start;
						break;
					}
					// Prevent infinite loops by checking if position has advanced
					if self.position <= start {
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
					let start = self.position;
					// Attempt to parse the delimiter
					let delimiter_nodes = self.eval(&*right)?;
					// If it fails, break the loop
					if delimiter_nodes.is_none() {
						self.position = start; // Technically unnecessary as a failure would leave position unchanged, but just to be consistent
						break;
					}
					// Attempt to parse the next expression
					let expression_nodes = self.eval(&*left)?;
					// If the next expression fails, break the loop
					if expression_nodes.is_none() {
						self.position = start;
						break;
					}
					// Prevent infinite loops by checking if position has advanced
					if self.position <= start {
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

				let mut last_pos = self.position;
				while let Some(new_nodes) = self.eval(&*expr)? {
					nodes.as_mut().unwrap().extend(new_nodes);
					if self.position == last_pos {
						break;
					}
					last_pos = self.position;
				}

				Ok(nodes)
			},
			Expression::RepeatZero(expr) => {
				let mut nodes = self.eval(&*expr)?;
				if nodes.is_none() { return Ok(Some(vec![])); }

				let mut last_pos = self.position;
				while let Some(new_nodes) = self.eval(&*expr)? {
					nodes.as_mut().unwrap().extend(new_nodes);
					if self.position == last_pos {
						break;
					}
					last_pos = self.position;
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
		let mut longest_nodes = None;

		for expr in expressions.iter() {
			let result = self.eval(&expr)?;
			let new_end_pos = self.position;
			self.position = start_pos; // Reset position to start for each expression evaluation
			if new_end_pos > longest_end || longest_nodes.is_none() {
				longest_end = new_end_pos;
				longest_nodes = result;
			}
		}
		if longest_nodes.is_none() {
			self.position = start_pos; // Reset position if no matches found
		} else {
			self.position = longest_end; // Update position to the end of the longest match
		}
		Ok(longest_nodes)
	}


	fn call_rule(&mut self, rule_name: &str, protected: bool) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		match rule_name {
			// Regular Rules
			"divide" => self._divide(),
			"private_keyword" => self._private_keyword(),
			"while_statement" => self._while_statement(),
			"type" => self._type(),
			"float_value" => self._float_value(),
			"integer_type" => self._integer_type(),
			"exponent" => self._exponent(),
			"return_statement" => self._return_statement(),
			"general_type" => self._general_type(),
			"dictionary_type" => self._dictionary_type(),
			"function_call" => self._function_call(),
			"index_keyword" => self._index_keyword(),
			"variable_increment" => self._variable_increment(),
			"dictionary_lookup" => self._dictionary_lookup(),
			"identifier" => self._identifier(),
			"repeat_statement" => self._repeat_statement(),
			"function_definition" => self._function_definition(),
			"list_lookup" => self._list_lookup(),
			"less_than_equals" => self._less_than_equals(),
			"comment" => self._comment(),
			"greater_than" => self._greater_than(),
			"greater_than_equals" => self._greater_than_equals(),
			"input_keyword" => self._input_keyword(),
			"not" => self._not(),
			"nothing_value" => self._nothing_value(),
			"file" => self._file(),
			"comparison" => {
				if protected {
					self.circular_wrapper("comparison".to_string())
				}
				else {
					self._comparison()
				}
			}
			"or" => self._or(),
			"less_than" => self._less_than(),
			"string_value" => self._string_value(),
			"times" => self._times(),
			"output_keyword" => self._output_keyword(),
			"float_type" => self._float_type(),
			"string_type" => self._string_type(),
			"integer_value" => self._integer_value(),
			"indent" => self._indent(),
			"plus" => self._plus(),
			"line_delimiter" => self._line_delimiter(),
			"if_statement" => self._if_statement(),
			"boolean_expression" => {
				if protected {
					self.circular_wrapper("boolean_expression".to_string())
				}
				else {
					self._boolean_expression()
				}
			}
			"equals" => self._equals(),
			"variable_set" => self._variable_set(),
			"list_type" => self._list_type(),
			"minus" => self._minus(),
			"pass_statement" => self._pass_statement(),
			"conversion" => {
				if protected {
					self.circular_wrapper("conversion".to_string())
				}
				else {
					self._conversion()
				}
			}
			"modulus" => self._modulus(),
			"not_equals" => self._not_equals(),
			"and" => self._and(),
			"boolean_type" => self._boolean_type(),
			"boolean_value" => self._boolean_value(),
			"new_lines" => self._new_lines(),
			"object_create" => self._object_create(),
			"equation" => {
				if protected {
					self.circular_wrapper("equation".to_string())
				}
				else {
					self._equation()
				}
			}
			"public_keyword" => self._public_keyword(),
			"variable_definition" => self._variable_definition(),
			// Meta Rules
			"function_argument" => {
				if protected {
					self.circular_wrapper("function_argument".to_string())
				}
				else {
					self._function_argument()
				}
			}
			"line" => return self._line(),
			"function_keyword" => return self._function_keyword(),
			"single_type" => return self._single_type(),
			"boolean_operator" => return self._boolean_operator(),
			"variable_value" => {
				if protected {
					self.circular_wrapper("variable_value".to_string())
				}
				else {
					self._variable_value()
				}
			}
			"variable_update" => return self._variable_update(),
			"lines" => return self._lines(),
			"control_statement" => return self._control_statement(),
			"compound_type" => return self._compound_type(),
			"block_line" => return self._block_line(),
			"math_operator" => return self._math_operator(),
			"member" => return self._member(),
			"value" => {
				if protected {
					self.circular_wrapper("value".to_string())
				}
				else {
					self._value()
				}
			}
			"comparator_operator" => return self._comparator_operator(),
			"variable_keyword" => return self._variable_keyword(),
			"variable_increment_value" => {
				if protected {
					self.circular_wrapper("variable_increment_value".to_string())
				}
				else {
					self._variable_increment_value()
				}
			}

			_ => Err(format!("Unknown rule: {}", rule_name).into()),
		}
	}


	fn _divide(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 2] = [
			StringLiteral("/"),
			StringLiteral("divided by"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("divide".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _private_keyword(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("private"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("private_keyword".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _while_statement(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral("while"), Rule("boolean_expression")), StringLiteral(":")), Rule("new_lines")), Expression::repeat_one(Rule("block_line"))),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("while_statement".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _type(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 2] = [
			Rule("compound_type"),
			Rule("single_type"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("type".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _float_value(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			RegexLiteral(r#"^[+-]?[0-9]+\.[0-9]+"#),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("float_value".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _integer_type(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("integer"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("integer_type".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _exponent(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("^"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("exponent".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _return_statement(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(StringLiteral("return"), Rule("line_delimiter")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("return_statement".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _general_type(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("general"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("general_type".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _dictionary_type(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(Expression::and(Rule("single_type"), StringLiteral("-")), Rule("single_type")), StringLiteral("-")), StringLiteral("dictionary")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("dictionary_type".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _function_call(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(StringLiteral("call"), Rule("identifier")), Expression::optional(Expression::and(StringLiteral("with"), Expression::or(Rule("function_argument"), Expression::delimit_repeat_one(Rule("function_argument"), StringLiteral(",")))))), Rule("line_delimiter")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("function_call".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _index_keyword(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("index"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("index_keyword".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _variable_increment(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral("change"), Rule("identifier")), StringLiteral("by")), Rule("variable_increment_value")), Rule("line_delimiter")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("variable_increment".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _dictionary_lookup(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(StringLiteral("lookup"), Rule("value")), StringLiteral("in")), Rule("identifier")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("dictionary_lookup".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _identifier(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			RegexLiteral(r#"^[a-z][a-z0-9_]*"#),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("identifier".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _repeat_statement(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral("repeat"), Rule("integer_value")), StringLiteral("times")), StringLiteral(":")), Rule("new_lines")), Expression::repeat_one(Rule("block_line"))),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("repeat_statement".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _function_definition(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral("create"), Expression::repeat_zero(Rule("function_keyword"))), StringLiteral("function")), Rule("identifier")), StringLiteral(":")), Rule("new_lines")), Expression::repeat_one(Rule("block_line"))),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("function_definition".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _list_lookup(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(StringLiteral("lookup"), Rule("integer_value")), StringLiteral("in")), Rule("identifier")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("list_lookup".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _less_than_equals(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("<="),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("less_than_equals".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _comment(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(StringLiteral("["), RegexLiteral(r#"^(?:[^\]]|\\\])*"#)), StringLiteral("]")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("comment".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _greater_than(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral(">"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("greater_than".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _greater_than_equals(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral(">="),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("greater_than_equals".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _input_keyword(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("input"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("input_keyword".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _not(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("not"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("not".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _nothing_value(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("nothing"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("nothing_value".to_string(), Vec::new(), start_pos);
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

	fn _comparison(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 5] = [
			Expression::and(Expression::and(Rule("comparison"), Rule("comparator_operator")), Rule("comparison")),
			Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral("("), Rule("comparison")), StringLiteral(")")), Rule("comparator_operator")), Rule("comparison")),
			Expression::and(Expression::and(StringLiteral("("), Rule("comparison")), StringLiteral(")")),
			Rule("equation"),
			Rule("nothing_value"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("comparison".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _or(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("or"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("or".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _less_than(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("<"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("less_than".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _string_value(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(StringLiteral("\""), RegexLiteral(r#"^([^"\\]|\\.)*"#)), StringLiteral("\"")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("string_value".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _times(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 2] = [
			StringLiteral("*"),
			StringLiteral("times"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("times".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _output_keyword(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("output"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("output_keyword".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _float_type(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("float"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("float_type".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _string_type(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("string"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("string_type".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _integer_value(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			RegexLiteral(r#"^[+-]?[0-9]+"#),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("integer_value".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _indent(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			RegexLiteral(r#"^[\t ]*"#),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("indent".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _plus(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 2] = [
			StringLiteral("+"),
			StringLiteral("plus"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("plus".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _line_delimiter(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(StringLiteral("."), Rule("new_lines")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("line_delimiter".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _if_statement(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral("if"), Rule("boolean_expression")), StringLiteral(":")), Rule("new_lines")), Expression::repeat_one(Rule("block_line"))),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("if_statement".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _boolean_expression(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 5] = [
			Expression::and(Expression::and(Rule("boolean_expression"), Rule("boolean_operator")), Rule("boolean_expression")),
			Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral("("), Rule("boolean_expression")), StringLiteral(")")), Rule("boolean_operator")), Rule("boolean_expression")),
			Expression::and(Expression::and(StringLiteral("("), Rule("boolean_expression")), StringLiteral(")")),
			Rule("comparison"),
			Rule("boolean_value"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("boolean_expression".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _equals(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 2] = [
			StringLiteral("=="),
			StringLiteral("is"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("equals".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _variable_set(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral("set"), Rule("identifier")), StringLiteral("to")), Rule("variable_value")), Rule("line_delimiter")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("variable_set".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _list_type(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Rule("single_type"), StringLiteral("-")), StringLiteral("list")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("list_type".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _minus(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 2] = [
			StringLiteral("-"),
			StringLiteral("minus"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("minus".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _pass_statement(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(StringLiteral("pass"), Rule("line_delimiter")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("pass_statement".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _conversion(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Rule("value"), StringLiteral("as")), Rule("type")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("conversion".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _modulus(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 2] = [
			StringLiteral("%"),
			StringLiteral("mod"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("modulus".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _not_equals(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 2] = [
			StringLiteral("!="),
			StringLiteral("is not"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("not_equals".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _and(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("and"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("and".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _boolean_type(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("boolean"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("boolean_type".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _boolean_value(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 2] = [
			StringLiteral("true"),
			StringLiteral("false"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("boolean_value".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _new_lines(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			RegexLiteral(r#"^(\r?\n)*"#),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("new_lines".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _object_create(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(StringLiteral("new"), Rule("identifier")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("object_create".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _equation(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 6] = [
			Expression::and(Expression::and(Rule("equation"), Rule("math_operator")), Rule("equation")),
			Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral("("), Rule("equation")), StringLiteral(")")), Rule("math_operator")), Rule("equation")),
			Expression::and(Expression::and(StringLiteral("("), Rule("equation")), StringLiteral(")")),
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

	fn _public_keyword(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			StringLiteral("public"),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("public_keyword".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}

	fn _variable_definition(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let start_pos = self.position;
		let expressions: [Expression; 1] = [
			Expression::and(Expression::and(Expression::and(Expression::and(Expression::and(Expression::and(Expression::and(StringLiteral("create"), Expression::repeat_zero(Rule("variable_keyword"))), Rule("type")), StringLiteral("variable")), Rule("identifier")), StringLiteral("with")), Rule("variable_value")), Rule("line_delimiter")),
		];

		if let Some(matches) = self.get_longest_expression_match(&expressions)? {
			let mut node = Node::Rule("variable_definition".to_string(), Vec::new(), start_pos);
			node.set_children(matches);
			return Ok(Some(vec![node]));
		}

		Ok(None)
	}


	fn _function_argument(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 2] = [
			Rule("equation"),
			Rule("object_create"),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _line(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 1] = [
			Expression::and(Rule("indent"), Rule("member")),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _function_keyword(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 2] = [
			Rule("public_keyword"),
			Rule("private_keyword"),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _single_type(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 6] = [
			Rule("integer_type"),
			Rule("float_type"),
			Rule("string_type"),
			Rule("boolean_type"),
			Rule("general_type"),
			Rule("identifier"),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _boolean_operator(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 3] = [
			Rule("and"),
			Rule("or"),
			Rule("not"),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _variable_value(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 3] = [
			Rule("equation"),
			Rule("function_call"),
			Rule("object_create"),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _variable_update(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 2] = [
			Rule("variable_set"),
			Rule("variable_increment"),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _lines(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 1] = [
			Expression::repeat_zero(Rule("line")),
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

	fn _compound_type(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 2] = [
			Rule("list_type"),
			Rule("dictionary_type"),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _block_line(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 1] = [
			Expression::and(Rule("indent"), Expression::or(Expression::or(Expression::or(Expression::or(Expression::or(Expression::or(Rule("comment"), Rule("variable_definition")), Rule("variable_update")), Rule("function_call")), Rule("control_statement")), Rule("return_statement")), Rule("pass_statement"))),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _math_operator(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 6] = [
			Rule("plus"),
			Rule("minus"),
			Rule("times"),
			Rule("divide"),
			Rule("modulus"),
			Rule("exponent"),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _member(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 3] = [
			Expression::and(Rule("comment"), Rule("new_lines")),
			Rule("variable_definition"),
			Rule("function_definition"),
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
			Rule("nothing_value"),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _comparator_operator(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 6] = [
			Rule("equals"),
			Rule("not_equals"),
			Rule("less_than"),
			Rule("less_than_equals"),
			Rule("greater_than"),
			Rule("greater_than_equals"),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _variable_keyword(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 5] = [
			Rule("public_keyword"),
			Rule("private_keyword"),
			Rule("input_keyword"),
			Rule("output_keyword"),
			Rule("index_keyword"),
		];

		self.get_longest_expression_match(&expressions)
	}

	fn _variable_increment_value(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		let expressions: [Expression; 2] = [
			Rule("equation"),
			Rule("function_call"),
		];

		self.get_longest_expression_match(&expressions)
	}

}