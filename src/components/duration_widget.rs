use crate::{
	project_tracker::Message,
	styles::{rounded_container_style, SMALL_HORIZONTAL_PADDING},
};
use iced::{
	widget::{container, text, Text},
	Element,
};
use project_tracker_core::duration_str;
use std::time::Duration;

pub fn duration_text(duration: Duration) -> Text<'static> {
	text(duration_str(duration))
}

pub fn duration_widget(duration: Duration) -> Element<'static, Message> {
	container(duration_text(duration))
		.padding(SMALL_HORIZONTAL_PADDING)
		.style(rounded_container_style)
		.into()
}
