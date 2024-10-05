use crate::{
	core::SerializableColor,
	project_tracker::Message,
	styles::{tooltip_container_style, SMALL_TEXT_SIZE},
};
use iced::{
	border::rounded,
	widget::{text, tooltip, tooltip::Position},
	Color, Element, Length,
};
use iced_aw::{quad::Quad, widgets::InnerBounds};
use serde::{Deserialize, Serialize};

pub const TASK_TAG_QUAD_HEIGHT: f32 = 5.0;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
		Self { name, color }
	}

	pub fn view(&self) -> Element<Message> {
		let color: Color = self.color.into();
		tooltip(
			Quad {
				width: Length::Fixed(50.0),
				height: Length::Fixed(TASK_TAG_QUAD_HEIGHT),
				inner_bounds: InnerBounds::Ratio(1.0, 1.0),
				quad_color: color.into(),
				quad_border: rounded(f32::MAX),
				..Default::default()
			},
			text(&self.name).size(SMALL_TEXT_SIZE),
			Position::Top,
		)
		.gap(5)
		.style(tooltip_container_style)
		.into()
	}
}
