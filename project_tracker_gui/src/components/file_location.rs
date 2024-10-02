use crate::{
	components::horizontal_scrollable,
	project_tracker::UiMessage,
	styles::{
		rounded_container_style, secondary_button_style_default, tooltip_container_style, GAP,
		SMALL_HORIZONTAL_PADDING, SMALL_TEXT_SIZE,
	},
};
use iced::{
	widget::{button, container, text, tooltip, tooltip::Position, Scrollable},
	Element,
};
use std::path::{Path, PathBuf};

pub fn filepath_widget<'a>(filepath: PathBuf) -> Scrollable<'a, UiMessage> {
	horizontal_scrollable(
		container(text(filepath.to_string_lossy().to_string()))
			.style(rounded_container_style)
			.padding(SMALL_HORIZONTAL_PADDING),
	)
}

pub fn file_location<'a>(filepath: PathBuf) -> Element<'a, UiMessage> {
	let parent_filepath = filepath.parent().map(Path::to_path_buf);

	horizontal_scrollable(
		tooltip(
			button(text(filepath.to_string_lossy().to_string()))
				.on_press_maybe(parent_filepath.map(UiMessage::OpenFolderLocation))
				.padding(SMALL_HORIZONTAL_PADDING)
				.style(secondary_button_style_default),
			text("Open folder location").size(SMALL_TEXT_SIZE),
			Position::Bottom,
		)
		.gap(GAP)
		.style(tooltip_container_style),
	)
	.into()
}
