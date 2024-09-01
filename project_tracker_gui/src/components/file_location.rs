use std::path::Path;
use iced::{theme, widget::{button, container, text, tooltip, tooltip::Position, Scrollable}, Element};
use crate::{project_tracker::UiMessage, components::horizontal_scrollable, styles::{RoundedContainerStyle, TooltipContainerStyle, SecondaryButtonStyle, GAP, SMALL_HORIZONTAL_PADDING, SMALL_TEXT_SIZE}};

pub fn filepath_widget(filepath: &Path) -> Scrollable<'static, UiMessage> {
	horizontal_scrollable(
		container(
			text(filepath.display())
		)
		.style(theme::Container::Custom(Box::new(RoundedContainerStyle)))
		.padding(SMALL_HORIZONTAL_PADDING)
	)
}

pub fn file_location(filepath: &Path) -> Element<'static, UiMessage> {
	let parent_filepath = filepath
		.parent()
		.map(Path::to_path_buf);

	horizontal_scrollable(
		tooltip(
			button(
				text(filepath.display())
			)
			.on_press_maybe(parent_filepath.map(UiMessage::OpenFolderLocation))
			.padding(SMALL_HORIZONTAL_PADDING)
			.style(theme::Button::custom(SecondaryButtonStyle::default())),

			text("Open folder location")
				.size(SMALL_TEXT_SIZE),

			Position::Bottom
		)
		.gap(GAP)
		.style(theme::Container::Custom(Box::new(TooltipContainerStyle)))
	)
	.into()
}