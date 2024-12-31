use crate::{
	components::{horizontal_scrollable, open_folder_location_button},
	project_tracker::Message,
	styles::{rounded_container_style, SMALL_HORIZONTAL_PADDING},
};
use iced::{
	widget::{container, text, Scrollable},
	Element,
};
use std::path::{Path, PathBuf};

pub fn filepath_widget<'a>(filepath: PathBuf) -> Scrollable<'a, Message> {
	horizontal_scrollable(
		container(text(filepath.to_string_lossy().to_string()))
			.style(rounded_container_style)
			.padding(SMALL_HORIZONTAL_PADDING),
	)
}

pub fn file_location<'a>(filepath: PathBuf) -> Element<'a, Message> {
	let parent_filepath = filepath.parent().map(Path::to_path_buf);

	horizontal_scrollable(open_folder_location_button(filepath, parent_filepath)).into()
}
