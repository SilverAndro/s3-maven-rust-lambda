use std::collections::HashMap;

pub struct Layer {
	children: HashMap<String, Box<Layer>>,

	pub packages: Vec<String>,
	pub files: Vec<String>
}

impl Layer {
	pub fn new() -> Layer {
		return Layer {
			children: HashMap::new(),
			packages: Vec::new(),
			files: Vec::new()
		}
	}

	pub fn has_children(&self, ids: &Vec<&str>, index: usize) -> bool {
		if index >= ids.len() {
			return true
		}

		let has_child = self.children.contains_key(ids[index]);
		return if has_child {
			self.children
				.get(ids[index])
				.expect("Failed to get child after just checking it exists")
				.has_children(ids, index + 1)
		} else {
			false
		}
	}

	pub fn get_or_compute_layer(&mut self, id: &str) -> &mut Layer {
		return self.children.entry(String::from(id))
			.or_insert(Box::from(Layer::new()))
	}

	pub fn descend(&self, ids: &Vec<&str>, index: usize) -> &Layer {
		if index >= ids.len() {
			return self
		}

		let child = self.children
			.get(ids[index])
			.expect("Call to descent requested id that does not exist in the path");
		return child.descend(ids, index + 1)
	}

	pub fn populate(&mut self, ids: &Vec<&str>, index: usize) -> &mut Layer {
		if index >= ids.len() {
			return self
		}

		self.packages.push(String::from(ids[index]));
		self.packages.dedup();

		let child = self.children.entry(String::from(ids[index]))
			.or_insert(Box::from(Layer::new()));
		return child.populate(ids, index + 1)
	}
}