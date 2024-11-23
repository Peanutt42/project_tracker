use crate::{
	project_tracker::Message,
	styles::{rounded_container_style, SMALL_HORIZONTAL_PADDING},
};
use iced::{
	widget::{container, text, Text},
	Element,
};
use pretty_duration::{pretty_duration, PrettyDurationLabels, PrettyDurationOptions};
use std::time::Duration;

pub fn round_duration_to_seconds(duration: Duration) -> Duration {
	Duration::from_secs(duration.as_secs())
}

pub fn duration_str(duration: Duration) -> String {
	pretty_duration(
		&duration,
		Some(PrettyDurationOptions {
			output_format: None,
			singular_labels: Some(PrettyDurationLabels {
				year: "year",
				month: "month",
				day: "day",
				hour: "h",
				minute: "min",
				second: "s",
				millisecond: "ms",
			}),
			plural_labels: Some(PrettyDurationLabels {
				year: "years",
				month: "months",
				day: "days",
				hour: "h",
				minute: "min",
				second: "s",
				millisecond: "ms",
			}),
		}),
	)
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
