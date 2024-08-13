use iced::{theme, widget::{container, container::Id, Space}, Element, Length, Padding};
use crate::{project_tracker::UiMessage, styles::{InBetweenDropzoneContainerStyle, SPACING_AMOUNT}};

const DROPZONE_HEIGHT: f32 = 2.0;
const DROPZONE_PADDING: f32 = (SPACING_AMOUNT as f32 - DROPZONE_HEIGHT) / 2.0;

pub fn in_between_dropzone(id: Id, highlight: bool) -> Element<'static, UiMessage> {
	container(
		container(
			container(
				Space::new(Length::Fill, DROPZONE_HEIGHT)
			)
			.style(theme::Container::Custom(Box::new(InBetweenDropzoneContainerStyle{ highlight })))
		)
		.padding(Padding {
			top: DROPZONE_PADDING,
			bottom: DROPZONE_PADDING,
			..Padding::ZERO
		})
	)
	.id(id)
	.into()
}