use crate::{
	core::{DateFormatting, SerializableDate},
	project_tracker::Message,
	styles::{rounded_container_style, SMALL_HORIZONTAL_PADDING},
};
use chrono::NaiveDate;
use iced::{
	widget::{container, text, text::Style, Text}, Element, Theme
};
use iced_aw::date_picker::Date;

pub fn date_text(date: &SerializableDate, formatting: DateFormatting) -> Text<'static> {
	text(formatting.format(date))
}

pub fn days_left_widget(date: SerializableDate, is_task_done: bool) -> Element<'static, Message> {
	let today: NaiveDate = Date::today().into();
	let date: Date = date.into();
	let date_naive: NaiveDate = date.into();
	let days_left = date_naive.signed_duration_since(today).num_days();

	let action_needed_text_style = if is_task_done {
		|_theme: &Theme| -> Style { Style { color: None } }
	}
	else {
		|theme: &Theme| -> Style {
			Style {
				color: Some(theme.extended_palette().danger.base.color)
			}
		}
	};

	container(match days_left {
		0 => text("due today")
			.style(action_needed_text_style),
		1 => text("due tomorrow"),
		-1 => text("due yesterday").style(action_needed_text_style),
		_ => {
			if days_left > 0 {
				text(format!("due in {days_left} days"))
			} else {
				text(format!("due {} days ago", -days_left)).style(action_needed_text_style)
			}
		}
	})
	.padding(SMALL_HORIZONTAL_PADDING)
	.style(rounded_container_style)
	.into()
}
