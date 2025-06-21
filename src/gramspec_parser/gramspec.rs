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