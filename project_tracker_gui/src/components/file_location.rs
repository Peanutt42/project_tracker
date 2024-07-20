use std::path::Path;
use iced::{widget::{row, text}, Alignment, Element};
use crate::{project_tracker::UiMessage, styles::SPACING_AMOUNT};
use crate::components::open_location_button;

pub fn file_location(filepath: &Path) -> Element<'static, UiMessage> {
	row![
		text(
			format!("{}", filepath.display())
		),
		open_location_button(filepath.parent().map(|folder| folder.to_path_buf())),
	]
	.align_items(Alignment::Center)
	.spacing(SPACING_AMOUNT)
	.into()
}