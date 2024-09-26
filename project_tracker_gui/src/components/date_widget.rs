use chrono::NaiveDate;
use iced::{widget::{container, text, Text}, Element};
use iced_aw::date_picker::Date;
use crate::{core::{DateFormatting, SerializableDate}, project_tracker::UiMessage, styles::{rounded_container_style, SMALL_HORIZONTAL_PADDING}};

pub fn date_text(date: &SerializableDate, formatting: DateFormatting) -> Text<'static> {
	text(formatting.format(date))
}

pub fn days_left_widget(date: SerializableDate) -> Element<'static, UiMessage> {
	let today: NaiveDate = Date::today().into();
	let date: Date = date.into();
	let date_naive: NaiveDate = date.into();
	let days_left = date_naive.signed_duration_since(today).num_days();

	container(
		match days_left {
			0 => text("due today"),
			1 => text("due tomorrow"),
			-1 => text("due yesterday"),
			_ => {
				if days_left > 0 {
					text(format!("due in {days_left} days"))
				}
				else {
					text(format!("due {} days ago", -days_left))
				}
			}
		}
	)
	.padding(SMALL_HORIZONTAL_PADDING)
	.style(rounded_container_style)
	.into()
}