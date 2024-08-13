use std::collections::hash_map::{HashMap, Values, ValuesMut};
use std::cmp::Eq;
use std::hash::Hash;
use std::slice::Iter;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderedHashMap<K, V> where K: Copy + Eq + Hash, V: Eq {
	hash_map: HashMap<K, V>,
	order: Vec<K>,
}

impl<K, V> OrderedHashMap<K, V>
	where K: Copy + Eq + Hash, V: Eq
{
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

	pub fn get_at_order(&self, order: usize) -> Option<&V> {
		match self.order.get(order) {
			Some(key) => self.hash_map.get(key),
			None => None,
		}
	}

	pub fn get_key_at_order(&self, order: usize) -> Option<&K> {
		self.order.get(order)
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

	pub fn move_to_end(&mut self, key: &K) {
		if let Some(order) = self.get_order(key) {
			for i in order..self.order.len()-1 {
				self.order.swap(i, i + 1);
			}
		}
	}

	/// Moves item with `key` before the item with `other_key`.
	/// Therefore `get_order(key) = get_order(other_key) - 1` must be true
	pub fn move_before_other(&mut self, key: K, other_key: K) {
		if let Some(order) = self.get_order(&key) {
			if let Some(other_order) = self.get_order(&other_key) {
				// already before other
				if order == other_order - 1 {
					return;
				}

				if order < other_order {
					for i in order..other_order-1 {
						self.order.swap(i, i + 1);
					}
				}
				else {
					for i in (other_order..order).rev() {
						self.order.swap(i, i + 1);
					}
				}
			}
		}
	}

	pub fn move_before_other_with_order(&mut self, order: usize, other_order: usize) {
		// already before other
		if order == other_order - 1 {
			return;
		}

		if order < other_order {
			for i in order..other_order-1 {
				self.order.swap(i, i + 1);
			}
		}
		else {
			for i in (other_order..order).rev() {
				self.order.swap(i, i + 1);
			}
		}
	}

	pub fn swap_order(&mut self, key_a: &K, key_b: &K) {
		if let Some(order_a) = self.get_order(key_a) {
			if let Some(order_b) = self.get_order(key_b) {
				self.order.swap(order_a, order_b);
			}
		}
	}

	pub fn insert(&mut self, key: K, value: V) {
		self.hash_map.insert(key, value);
		self.order.push(key);
	}

	pub fn remove(&mut self, key: &K) -> Option<V> {
		let value = self.hash_map.remove(key);
		if let Some(index) = self.get_order(key) {
			self.order.remove(index);
		}
		value
	}

	pub fn contains_key(&self, key: &K) -> bool {
		self.hash_map.contains_key(key)
	}

	pub fn get(&self, key: &K) -> Option<&V> {
		self.hash_map.get(key)
	}

	pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
		self.hash_map.get_mut(key)
	}

	pub fn move_to(&mut self, key: K, order: usize) {
		if let Some(old_order) = self.get_order(&key) {
			self.order.remove(old_order);
			self.order.insert(order, key);
		}
	}

	pub fn len(&self) -> usize {
		self.order.len()
	}

	pub fn is_empty(&self) -> bool {
		self.order.is_empty()
	}

	pub fn iter(&self) -> OrderedHashMapIter<K, V> {
		OrderedHashMapIter { order_iter: self.order.iter(), hash_map: &self.hash_map }
	}

	pub fn keys(&self) -> Iter<K> {
		self.order.iter()
	}

	pub fn values(&self) -> Values<K, V> {
		self.hash_map.values()
	}

	pub fn values_mut(&mut self) -> ValuesMut<K, V> {
		self.hash_map.values_mut()
	}
}

impl<K, V> Default for OrderedHashMap<K, V> where K: Copy + Eq + Hash, V: Eq {
	fn default() -> Self {
		Self::new()
	}
}

pub struct OrderedHashMapIter<'a, K, V>
	where K: Eq + Copy + Hash, V: Eq
{
	order_iter: Iter<'a, K>,
	hash_map: &'a HashMap<K, V>,
}

impl<'a, K, V> Iterator for OrderedHashMapIter<'a, K, V>
	where K: Eq + Copy + Hash, V: Eq
{
	type Item = (K, &'a V);

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(key) = self.order_iter.next() {
			self.hash_map.get(key).map(|value| (*key, value))
		}
		else {
			None
		}
	}
}