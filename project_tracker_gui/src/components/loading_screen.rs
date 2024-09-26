use iced::{widget::container, Element, Length::{self, Fill}};
use iced_aw::Spinner;
use crate::project_tracker::UiMessage;

pub fn loading_screen() -> Element<'static, UiMessage> {
	container(
		Spinner::new()
			.width(Length::Fixed(75.0))
			.height(Length::Fixed(75.0)).circle_radius(3.0)
	)
	.center_x(Fill)
	.center_y(Fill)
	.into()
}