use crate::{
	project_tracker::UiMessage,
	styles::{rounded_container_style, SMALL_HORIZONTAL_PADDING},
};
use iced::{
	widget::{container, text, Text},
	Element,
};
use pretty_duration::{pretty_duration, PrettyDurationLabels, PrettyDurationOptions};
use std::{borrow::Cow, time::Duration};

pub fn duration_text(duration: Cow<'_, Duration>) -> Text {
	text(pretty_duration(
		duration.as_ref(),
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
	))
}

pub fn duration_widget(duration: Cow<'_, Duration>) -> Element<'_, UiMessage> {
	container(duration_text(duration))
		.padding(SMALL_HORIZONTAL_PADDING)
		.style(rounded_container_style)
		.into()
}
