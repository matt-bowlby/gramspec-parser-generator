pub mod gramspec_config;
pub mod expression;

use std::collections::HashMap;
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

	pub fn add_rule(&mut self, name: String, expressions: Vec<Expression>) {
		self.rules.insert(name, expressions);
	}

	pub fn add_meta_rule(&mut self, name: String, expressions: Vec<Expression>) {
		self.meta_rules.insert(name, expressions);
	}
}