#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Node {
	Rule(String, Vec<Box<Node>>, usize),
	String(String, usize),
}

#[allow(dead_code)]
impl Node {
	pub fn append(&mut self, child: Node) {
		match self {
			Node::Rule(_, children, _) => children.push(Box::new(child)),
			_ => return,
		}
	}

	pub fn extend(&mut self, children: Vec<Node>) {
		// Only extend if the current node can be extended
		if !matches!(self, Node::Rule(_, _, _)) { return; }

		// Extend the current node with the provided children
		for child in children {
			self.append(child);
		}
	}

	pub fn get_end_pos(&self) -> usize {
		match self {
			Node::Rule(_, nodes, start_pos) => {
				for node in nodes {
					let end_pos = node.get_end_pos();
					if end_pos > *start_pos {
						return end_pos;
					}
				}
				*start_pos
			},
			Node::String(string, start_pos) => *start_pos + string.len(),
		}
	}
}