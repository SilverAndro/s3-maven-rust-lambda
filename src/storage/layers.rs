use std::collections::HashMap;

// Acts like a tree like structure
pub struct Layer {
	children: HashMap<String, Box<Layer>>,

	pub packages: Vec<String>,
	pub files: Vec<String>
}

impl Layer {
	pub fn new() -> Layer {
		Layer {
			children: HashMap::new(),
			packages: Vec::new(),
			files: Vec::new()
		}
	}

	pub fn has_children(&self, ids: &Vec<&str>, index: usize) -> bool {
		if index >= ids.len() {
			return !(self.packages.is_empty() && self.files.is_empty())
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

	fn get_or_compute_layer(&mut self, id: &str) -> &mut Layer {
		return self.children.entry(String::from(id))
			.or_insert(Box::from(Layer::new()))
	}

	pub fn descend(&self, ids: &Vec<&str>, index: usize) -> &Layer {
		if index >= ids.len() {
			return self
		}

		let child = self.children
			.get(ids[index])
			.unwrap_or_else(|| panic!("Call to descent requested id that does not exist in the path, was looking for {:?}", ids));
		return child.descend(ids, index + 1)
	}

	pub fn populate(&mut self, ids: &Vec<&str>, index: usize) -> &mut Layer {
		if index >= ids.len() {
			return self
		}

		self.packages.push(String::from(ids[index]));
		self.packages.dedup();

		let child = self.get_or_compute_layer(ids[index]);
		return child.populate(ids, index + 1)
	}
}

// Awful hack, but we only ever copy when building a specific layer, so we just return an empty vec
// of child layers to avoid issues
impl Clone for Layer {
	fn clone(&self) -> Self {
		Layer {
			children: Default::default(),
			packages: self.packages.clone(),
			files: self.files.clone(),
		}
	}
}