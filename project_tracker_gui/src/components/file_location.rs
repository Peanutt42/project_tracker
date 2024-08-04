use std::path::Path;
use iced::{theme, widget::{container, row, scrollable, text}, Alignment, Element, Padding};
use crate::{project_tracker::UiMessage, styles::{scrollable_horizontal_direction, ScrollableStyle, SCROLLBAR_WIDTH, SMALL_PADDING_AMOUNT, SPACING_AMOUNT}};
use crate::components::open_location_button;

pub fn filepath_widget(filepath: &Path) -> Element<'static, UiMessage> {
	scrollable(
		container(
			container(
				text(
					filepath.display()
				)
			)
			.style(theme::Container::Box)
		)
		.padding(Padding{ bottom: SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT, top: SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT, ..Padding::ZERO })
	)
	.direction(scrollable_horizontal_direction())
	.style(theme::Scrollable::custom(ScrollableStyle))
	.into()
}

pub fn file_location(filepath: &Path) -> Element<'static, UiMessage> {
	row![
		open_location_button(filepath.parent().map(|folder| folder.to_path_buf())),

		filepath_widget(filepath),
	]
	.align_items(Alignment::Center)
	.spacing(SPACING_AMOUNT)
	.into()
}