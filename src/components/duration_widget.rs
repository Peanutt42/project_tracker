use crate::{
	project_tracker::Message,
	styles::{rounded_container_style, SMALL_HORIZONTAL_PADDING},
};
use iced::{
	widget::{container, text, Text},
	Element,
};
use humantime::format_duration;
use std::time::Duration;

pub fn round_duration_to_seconds(duration: Duration) -> Duration {
	Duration::from_secs(duration.as_secs())
}

pub fn duration_to_minutes(duration: Duration) -> usize {
	duration.as_secs() as usize / 60
}

pub fn parse_duration_from_str(string: &str) -> Option<Duration> {
	humantime::parse_duration(string).ok()
}

pub fn duration_str(duration: Duration) -> String {
	format_duration(duration).to_string()
}

pub fn duration_text(duration: Duration) -> Text<'static> {
	text(duration_str(duration))
}

pub fn duration_widget(duration: Duration) -> Element<'static, Message> {
	container(duration_text(duration))
		.padding(SMALL_HORIZONTAL_PADDING)
		.style(rounded_container_style)
		.into()
}
