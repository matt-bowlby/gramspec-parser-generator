use std::fmt;
use std::error::Error;

use crate::parser::lang::Lang;
use crate::parser::node::Node;

use crate::parser::Parser;

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
			Expression::RegexLiteral(regex) => write!(f, "\"{}\"", regex),
			Expression::StringLiteral(string) => write!(f, "\'{}\'", string),
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
impl Expression {
	pub fn eval(&self, lang: &mut Lang, parser: &Parser) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
		match self {
			Expression::Rule(rule) => {
				if let Some(nodes) = parser.call_rule(rule, lang)? {
					Ok(Some(nodes))
				} else {
					Ok(None)
				}
			},
			Expression::RegexLiteral(regex) => lang.expect_regex(regex),
			Expression::StringLiteral(string) => lang.expect_string(string),
			Expression::Keyword(keyword) => lang.expect_keyword(keyword),
			Expression::Or(left, right) => {
				let start_pos = lang.position;
				let left_nodes = left.eval(lang, parser)?;
				let left_end = lang.position;
				lang.position = start_pos;
				let right_nodes = right.eval(lang, parser)?;
				let right_end = lang.position;

				if left_end > right_end {
					lang.position = left_end;
					return Ok(left_nodes);
				} else if right_end > left_end {
					lang.position = right_end;
					return Ok(right_nodes);
				} else {
					lang.position = start_pos;
					return Ok(None);
				}
			},
			Expression::And(left, right) => {
				let left_nodes = left.eval(lang, parser)?;
				if left_nodes.is_none() {
					return Ok(None);
				}
				let right_nodes = right.eval(lang, parser)?;
				if right_nodes.is_none() {
					return Ok(None);
				}
				let mut final_nodes = left_nodes.unwrap();
				final_nodes.extend(right_nodes.unwrap());
				Ok(Some(final_nodes))
			},
			Expression::DelimitRepeatOne(expression, delimiter) => {
				// Attempt to parse the first expression
				let nodes = expression.eval(lang, parser)?;
				// If the first expression fails, return an empty vector
				if nodes.is_none() {
					return Ok(None);
				}

				let mut nodes = nodes.unwrap();

				// Attempt to parse subsequent expressions with delimiters
				loop {
					// Attempt to parse the delimiter
					let delimiter_nodes = delimiter.eval(lang, parser)?;
					// If it fails, break the loop
					if delimiter_nodes.is_none() {
						break;
					}
					// Attempt to parse the next expression
					let expression_nodes = expression.eval(lang, parser)?;
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
				let nodes = left.eval(lang, parser)?;
				// If the first expression fails, return an empty vector
				if nodes.is_none() {
					return Ok(Some(vec![]));
				}

				let mut nodes = nodes.unwrap();

				// Attempt to parse subsequent expressions with delimiters
				loop {
					// Attempt to parse the delimiter
					let delimiter_nodes = right.eval(lang, parser)?;
					// If it fails, break the loop
					if delimiter_nodes.is_none() {
						break;
					}
					// Attempt to parse the next expression
					let expression_nodes = left.eval(lang, parser)?;
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
				let mut nodes = expr.eval(lang, parser)?;

				if nodes.is_none() { return Ok(None); }

				while let Some(new_nodes) = expr.eval(lang, parser)? {
					nodes.as_mut().unwrap().extend(new_nodes);
				}

				Ok(nodes)
			},
			Expression::RepeatZero(expr) => {
				let mut nodes = expr.eval(lang, parser)?;

				if nodes.is_none() { return Ok(Some(vec![])); }

				while let Some(new_nodes) = expr.eval(lang, parser)? {
					nodes.as_mut().unwrap().extend(new_nodes);
				}

				Ok(nodes)
			},
			Expression::Optional(expr) => {
				let mut nodes = expr.eval(lang, parser)?;

				if nodes.is_none() {
					nodes = Some(vec![]);
				}

				Ok(nodes)
			},
		}
	}
}