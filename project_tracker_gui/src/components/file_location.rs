use std::path::Path;
use iced::{theme, widget::{container, row, text}, Alignment, Element};
use crate::{project_tracker::UiMessage, styles::{capped_text, SPACING_AMOUNT}};
use crate::components::open_location_button;

pub fn file_location(filepath: &Path) -> Element<'static, UiMessage> {
	row![
		container(
			text(
				capped_text(&filepath.to_string_lossy(), 60)
			)
		)
		.style(theme::Container::Box),
		open_location_button(filepath.parent().map(|folder| folder.to_path_buf())),
	]
	.align_items(Alignment::Center)
	.spacing(SPACING_AMOUNT)
	.into()
}