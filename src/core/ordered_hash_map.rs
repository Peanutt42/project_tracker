use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderedHashMap<K: Copy + std::cmp::Eq + std::hash::Hash, V> {
	hash_map: HashMap<K, V>,
	pub order: Vec<K>,
}

impl<K: Copy + std::cmp::Eq + std::hash::Hash, V> OrderedHashMap<K, V> {
	pub fn new() -> Self {
		Self {
			hash_map: HashMap::new(),
			order: Vec::new(),
		}
	}

	pub fn get_order(&self, key: &K) -> Option<usize> {
		for (i, other_key) in self.order.iter().enumerate() {
			if other_key == key {
				return Some(i);
			}
		}
		None
	}

	pub fn move_up(&mut self, key: &K) {
		if let Some(index) = self.get_order(key) {
			if index != 0 {
				self.order.swap(index, index - 1);
			}
		}
	}

	pub fn move_down(&mut self, key: &K) {
		if let Some(index) = self.get_order(key) {
			if index != self.order.len() - 1 {
				self.order.swap(index, index + 1);
			}
		}
	}

	pub fn insert(&mut self, key: K, value: V) {
		self.hash_map.insert(key, value);
		self.order.push(key);
	}

	pub fn remove(&mut self, key: &K) {
		self.hash_map.remove(key);
		if let Some(index) = self.get_order(key) {
			self.order.remove(index);
		}
	}

	pub fn get(&self, key: &K) -> Option<&V> {
		self.hash_map.get(key)
	}

	pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
		self.hash_map.get_mut(key)
	}

	pub fn move_to_bottom(&mut self, key: &K) {
		if let Some(index) = self.get_order(key) {
			self.order.remove(index);
		}

		self.order.push(*key);
	}

	pub fn len(&self) -> usize {
		self.order.len()
	}

	pub fn iter(&self) -> std::slice::Iter<K> {
		self.order.iter()
	}

	pub fn values(&self) -> std::collections::hash_map::Values<K, V> {
		self.hash_map.values()
	}
}
