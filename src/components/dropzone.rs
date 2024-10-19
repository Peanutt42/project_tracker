use crate::{
	project_tracker::Message,
	styles::{in_between_dropzone_container_style, SPACING_AMOUNT},
};
use iced::{
	widget::{container, container::Id, Space},
	Element,
	Length::Fill,
	Padding,
};

const DROPZONE_HEIGHT: f32 = 3.5;
const DROPZONE_PADDING: f32 = (SPACING_AMOUNT as f32 - DROPZONE_HEIGHT) / 2.0;

pub fn in_between_dropzone(id: Id, highlight: bool) -> Element<'static, Message> {
	container(
		container(
			container(Space::new(Fill, DROPZONE_HEIGHT))
				.style(move |t| in_between_dropzone_container_style(t, highlight)),
		)
		.padding(Padding {
			top: DROPZONE_PADDING,
			bottom: DROPZONE_PADDING,
			..Padding::ZERO
		}),
	)
	.id(id)
	.into()
}
