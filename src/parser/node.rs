pub enum Node {
	Rule(String, Vec<Box<Node>>),
	String(String),
}

impl Node {
	pub fn append(&mut self, child: Node) -> Result<(), String> {
		match self {
			Node::Rule(_, children) => children.push(Box::new(child)),
			_ => return Err("Cannot append to a non-rule node".into()),
		}
		Ok(())
	}
}