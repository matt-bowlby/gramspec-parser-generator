pub mod gramspec_config;

use crate::gramspec::gramspec_config::GramSpecConfig;
use crate::node::{Node, NodeType};
use std::collections::{HashMap, HashSet};

pub struct GramSpec {
    pub rules: HashMap<String, Vec<Node>>,
    pub config: GramSpecConfig,
    pub meta_rules: HashMap<String, Vec<Node>>,
    pub discard_rules: HashMap<String, Vec<Node>>,
}

impl GramSpec {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            config: GramSpecConfig::new(),
            meta_rules: HashMap::new(),
            discard_rules: HashMap::new(),
        }
    }

    pub fn from(entry_node: Node) -> Self {
        let mut gramspec = GramSpec::new();

        for child in entry_node.children {
            match child.node_type {
                crate::node::NodeType::RuleDefinition => {
                    let (rule_name, alternatives) = gramspec.rule_from_node(&child).unwrap();
                    gramspec.rules.insert(rule_name, alternatives);
                }
                crate::node::NodeType::MetaRuleDefinition => {
                    let rule_name = child.value.unwrap();
                    let alternatives = child.children;
                    gramspec.meta_rules.insert(rule_name, alternatives);
                }
                crate::node::NodeType::DiscardRuleDefinition => {
                    let rule_name = child.value.unwrap();
                    let alternatives = child.children;
                    gramspec.discard_rules.insert(rule_name, alternatives);
                }
                crate::node::NodeType::ConfigDirective => {

                }
                _ => {}
            }
        }

        gramspec
    }

    pub fn get_alternatives(&self, rule_name: &str) -> Option<&Vec<Node>> {
        if let Some(rule_expression) = self.rules.get(rule_name) {
            return Some(rule_expression);
        }
        if let Some(meta_expression) = self.meta_rules.get(rule_name) {
            return Some(meta_expression);
        }
        if let Some(discard_expression) = self.discard_rules.get(rule_name) {
            return Some(discard_expression);
        }
        None
    }

    fn rule_from_node(&self, node: &Node) -> Option<(String, Vec<Node>)> {
        if node.node_type != NodeType::RuleDefinition {
            return None;
        }
        let rule_name = node.children[0].value.clone();
        let alternatives = node.iter_children().skip(1).find_map(|node| {if node.node_type == NodeType::Expressions {
            Some(node.children.clone())
        } else {
            None
        }});
        Some((rule_name.unwrap(), alternatives.unwrap_or_default()))
    }

    pub fn is_left_circular(&self, rule_name: &str) -> bool {
        if let Some(alternatives) = self.get_alternatives(rule_name) {
            for expr in alternatives {
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
        node: &Node,
        visited: &mut HashSet<String>,
    ) -> bool {
        match node.node_type {
            NodeType::Identifier => {
                let rule_name = node.value.as_ref().unwrap();

                if visited.contains(rule_name) || rule_name == original_rule {
                    return true;
                }

                visited.insert(rule_name.to_string());

                if let Some(expressions) = self.get_alternatives(&rule_name) {
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

            NodeType::String | NodeType::Regex => false,

            _ => {
                for child in &node.children {
                    if self.is_left_circular_expression(original_rule, child, visited) {
                        return true;
                    }
                }
                false
            }
        }
    }
}
