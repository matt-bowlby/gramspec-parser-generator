pub struct Lang {
	name: &'static str,
	content: String,
	position: usize,

	marked_positions: Vec<usize>,
}

impl Lang {
	pub fn new(name: &'static str, content: String) -> Self {
		Lang { name, position: 0, content, marked_positions: Vec::new() }
	}

	pub fn get_name(&self) -> &str {
		&self.name
	}

	pub fn set_content(&mut self, content: String) {
		self.position = 0;
		self.content = content;
	}

	pub fn get_content(&self) -> &str {
		&self.content
	}

	pub fn mark(&mut self) {
		self.marked_positions.push(self.position);
	}

	pub fn reset(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		self.position = self.marked_positions.pop().ok_or::<Box<dyn std::error::Error>>(
			format!("No marked position to reset to").into()
		)?;
		Ok(())
	}
}