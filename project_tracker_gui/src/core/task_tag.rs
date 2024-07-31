use iced_aw::{quad::Quad, widgets::InnerBounds};
use serde::{Deserialize, Serialize};
use iced::{Border, Color, Element, Length};
use crate::{core::SerializableColor, project_tracker::UiMessage};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TaskTagId(pub usize);

impl TaskTagId {
	pub fn generate() -> Self {
		Self(rand::random())
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TaskTag {
	pub name: String,
	pub color: SerializableColor,
}

impl TaskTag {
	pub fn new(name: String, color: SerializableColor) -> Self {
		Self {
			name,
			color,
		}
	}

	pub fn view(&self) -> Element<UiMessage> {
		let color: Color = self.color.into();
		Quad {
			width: Length::Fixed(50.0),
			height: Length::Fixed(5.0),
			inner_bounds: InnerBounds::Ratio(1.0, 1.0),
			quad_color: color.into(),
			quad_border: Border::with_radius(f32::MAX),
			..Default::default()
		}
		.into()
	}
}