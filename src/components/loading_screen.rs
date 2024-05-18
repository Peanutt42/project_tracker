use iced::{Element, widget::container, Length};
use iced_aw::Spinner;
use crate::project_tracker::UiMessage;

pub fn loading_screen() -> Element<'static, UiMessage> {
	container(
		Spinner::new()
			.width(Length::Fixed(75.0))
			.height(Length::Fixed(75.0)).circle_radius(3.0)
	)
	.width(Length::Fill)
	.height(Length::Fill)
	.center_x()
	.center_y()
	.into()
}