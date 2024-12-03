use crate::{
	project_tracker::Message,
	styles::{tooltip_container_style, SMALL_TEXT_SIZE},
	core::View,
	core::IcedColorConversion,
};
use iced::{
	border::rounded,
	widget::{text, tooltip, tooltip::Position}, Element, Length,
};
use iced_aw::{quad::Quad, widgets::InnerBounds};
use project_tracker_core::TaskTag;

pub const TASK_TAG_QUAD_HEIGHT: f32 = 5.0;

impl View for TaskTag {
	fn view(&self) -> Element<Message> {
		let color = self.color.to_iced_color();
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