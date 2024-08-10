use iced::{theme, widget::{container, text, Text}, Element};
use crate::{core::{DateFormatting, SerializableDate}, project_tracker::UiMessage, styles::{RoundedContainerStyle, SMALL_HORIZONTAL_PADDING}};

pub fn date_text(date: &SerializableDate, formatting: DateFormatting) -> Text<'static> {
	text(formatting.format(date))
}

pub fn date_widget(date: &SerializableDate, formatting: DateFormatting) -> Element<UiMessage> {
	container(
		date_text(date, formatting)
	)
	.padding(SMALL_HORIZONTAL_PADDING)
	.style(theme::Container::Custom(Box::new(RoundedContainerStyle)))
	.into()
}