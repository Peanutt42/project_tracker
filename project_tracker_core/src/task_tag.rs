use crate::SerializableColor;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskTagId(pub Uuid);

impl TaskTagId {
	pub fn generate() -> Self {
		Self(Uuid::new_v4())
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TaskTag {
	pub name: String,
	pub color: SerializableColor,
}

impl TaskTag {
	pub fn new(name: String, color: SerializableColor) -> Self {
		Self { name, color }
	}
}
