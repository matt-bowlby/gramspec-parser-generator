pub mod gramspec_config;
pub mod expression;

use std::collections::{HashMap, HashSet};
use crate::gramspec_parser::gramspec::expression::Expression;
use crate::gramspec_parser::gramspec::gramspec_config::GramSpecConfig;

pub struct GramSpec {
	pub rules: HashMap<String, Vec<Expression>>,
	pub config: GramSpecConfig,
	pub meta_rules: HashMap<String, Vec<Expression>>,
}

impl GramSpec {
	pub fn new() -> Self {
		Self {
			rules: HashMap::new(),
			config: GramSpecConfig::new(),
			meta_rules: HashMap::new(),
		}
	}

	pub fn get_expression(&self, rule_name: &str) -> Option<&Vec<Expression>> {
		if let Some(rule_expression) = self.rules.get(rule_name) {
			return Some(rule_expression);
		}
		if let Some(meta_expression) = self.meta_rules.get(rule_name) {
			return Some(meta_expression);
		}
		None
	}

	pub fn add_rule(&mut self, name: String, expressions: Vec<Expression>) {
		self.rules.insert(name, expressions);
	}

	pub fn add_meta_rule(&mut self, name: String, expressions: Vec<Expression>) {
		self.meta_rules.insert(name, expressions);
	}
	pub fn is_left_circular(&self, rule_name: &str) -> bool {
		if let Some(expressions) = self.get_expression(rule_name) {
			for expr in expressions {
				let mut visited = HashSet::new();
				if self.is_left_circular_expression(rule_name, expr, &mut visited) {
					return true;
				}
			}
		}
		false
	}

	fn is_left_circular_expression(
		&self,
		original_rule: &str,
		expr: &Expression,
		visited: &mut HashSet<String>
	) -> bool {
		match expr {
			Expression::RuleName(token) => {
				let rule_name = &token.value;

				if visited.contains(rule_name) {
					return rule_name == original_rule;
				}

				visited.insert(rule_name.to_string());

				if let Some(expressions) = self.get_expression(rule_name) {
					for expr in expressions {
						if self.is_left_circular_expression(original_rule, expr, visited) {
							visited.remove(rule_name);
							return true;
						}
					}
				}

				visited.remove(rule_name);
				false
			}

			Expression::StringLiteral(_) |
			Expression::RegexLiteral(_) |
			Expression::Keyword(_) => false,

			Expression::And(left, _right) => {
				self.is_left_circular_expression(original_rule, left, visited)
			}

			Expression::Or(left, right) => {
				self.is_left_circular_expression(original_rule, left, visited) ||
				self.is_left_circular_expression(original_rule, right, visited)
			}

			Expression::Optional(inner) => {
				self.is_left_circular_expression(original_rule, inner, visited)
			}

			Expression::RepeatZero(inner) => {
				self.is_left_circular_expression(original_rule, inner, visited)
			}

			Expression::RepeatOne(inner) => {
				self.is_left_circular_expression(original_rule, inner, visited)
			}

			Expression::DelimitRepeatZero(expr, _delim) => {
				self.is_left_circular_expression(original_rule, expr, visited)
			}

			Expression::DelimitRepeatOne(expr, _delim) => {
				self.is_left_circular_expression(original_rule, expr, visited)
			}
		}
	}
}