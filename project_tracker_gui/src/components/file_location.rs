use std::path::Path;
use iced::{theme, widget::{button, container, scrollable, text, tooltip, tooltip::Position}, Element, Length, Padding};
use crate::{project_tracker::UiMessage, styles::{scrollable_horizontal_direction, RoundedContainerStyle, TooltipContainerStyle, ScrollableStyle, SecondaryButtonStyle, GAP, SCROLLBAR_WIDTH, SMALL_HORIZONTAL_PADDING, SMALL_PADDING_AMOUNT, SMALL_TEXT_SIZE}};

pub fn filepath_widget(filepath: &Path) -> Element<'static, UiMessage> {
	scrollable(
		container(
			container(
				text(filepath.display())
			)
			.style(theme::Container::Custom(Box::new(RoundedContainerStyle)))
			.padding(SMALL_HORIZONTAL_PADDING)
		)
		.padding(Padding{
			top: SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT,
			bottom: SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT,
			..Padding::ZERO
		})
	)
	.width(Length::Fill)
	.direction(scrollable_horizontal_direction())
	.style(theme::Scrollable::custom(ScrollableStyle))
	.into()
}

pub fn file_location(filepath: &Path) -> Element<'static, UiMessage> {
	let parent_filepath = filepath
		.parent()
		.map(Path::to_path_buf);

	scrollable(
		container(
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
		.padding(Padding{
			top: SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT,
			bottom: SCROLLBAR_WIDTH + SMALL_PADDING_AMOUNT,
			..Padding::ZERO
		})
	)
	.direction(scrollable_horizontal_direction())
	.style(theme::Scrollable::custom(ScrollableStyle))
	.into()
}