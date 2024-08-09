use iced::{theme, widget::{container, text, Text}, Element};

use crate::{core::SerializableDate, project_tracker::UiMessage, styles::{RoundedContainerStyle, SMALL_HORIZONTAL_PADDING}};


// TODO: more formats (MM.DD.YY)
pub fn date_text(date: &SerializableDate) -> Text<'static> {
	text(format!("{}.{}.{}", date.day, date.month, date.year))
}

pub fn date_widget(date: &SerializableDate) -> Element<UiMessage> {
	container(
		date_text(date)
	)
	.padding(SMALL_HORIZONTAL_PADDING)
	.style(theme::Container::Custom(Box::new(RoundedContainerStyle)))
	.into()
}