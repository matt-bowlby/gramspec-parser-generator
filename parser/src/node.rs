#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    DelimitRepeatZero,
    Group,
    Identifier,
    Optional,
    NewLines,
    DelimitRepeatOne,
    MetaRuleDefinition,
    RepeatOne,
    DiscardRuleDefinition,
    DiscardValue,
    WhiteSpace,
    Comment,
    RepeatZero,
    And,
    RuleDefinition,
    String,
    ConfigDirective,
    File,
    Or,
    Regex,
    Expressions,
    MetaValue,
    _String,
    _Discard,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Node>,
    pub value: Option<String>,
    pub start_position: usize,
    pub end_position: usize,
}

#[allow(dead_code)]
impl Node {
    pub fn new(node_type: NodeType, children: Vec<Node>, value: Option<String>) -> Self {
        Node {
            node_type,
            children,
            value,
            start_position: 0,
            end_position: 0,
        }
    }

    pub fn iter_children(&self) -> std::slice::Iter<'_, Node> {
        self.children.iter()
    }

    pub fn formatted(&self, indent: usize, indent_string: &str) -> String {
        let mut result = String::new();
        let indent_str = indent_string.repeat(indent);
        result.push_str(&indent_str);
        if self.node_type != NodeType::_String {
            result.push_str(&format!("{:?}: ", self.node_type));
        }
        if let Some(val) = &self.value {
            result.push_str(&format!("\"{}\"", val.escape_debug()));
        }
        for child in &self.children {
            result.push_str(&format!("\n{}", child.formatted(indent + 1, indent_string)));
        }
        result
    }

    pub fn pretty_print(&self) {
        println!("{}", self.formatted(0, "    "));
    }

    pub(super) fn extend(&mut self, children: &Vec<Node>) {
        self.children.extend(children.iter().cloned());
    }

    pub(super) fn new_with_position(
        node_type: NodeType,
        children: Vec<Node>,
        value: Option<String>,
        start_position: usize,
        end_position: usize,
    ) -> Self {
        Node {
            node_type,
            children,
            value,
            start_position,
            end_position,
        }
    }

    pub(super) fn get_end_pos(&self) -> usize {
        if let Some(last_child) = self.children.last() {
            last_child.get_end_pos()
        } else {
            self.end_position
        }
    }

    pub(super) fn append(&mut self, child: Node) {
        self.children.push(child);
    }
}
