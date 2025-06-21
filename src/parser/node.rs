#[allow(dead_code)]
pub enum Node {
	Rule(String, Vec<Box<Node>>),
	String(String),
}

#[allow(dead_code)]
impl Node {
	pub fn append(&mut self, child: Node) {
		match self {
			Node::Rule(_, children) => children.push(Box::new(child)),
			_ => return,
		}
	}

	pub fn extend(&mut self, children: Vec<Node>) {
		// Only extend if the current node can be extended
		if !matches!(self, Node::Rule(_, _)) { return; }

		// Extend the current node with the provided children
		for child in children {
			self.append(child);
		}
	}
}