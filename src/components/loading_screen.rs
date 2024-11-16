use iced::{
	Element,
	Length,
};
use iced_aw::Spinner;

pub const LARGE_LOADING_SPINNER_SIZE: f32 = 75.0;
pub const SMALL_LOADING_SPINNER_SIZE: f32 = 25.0;

pub fn loading_screen<Message: 'static>(size: f32) -> Element<'static, Message> {
	Spinner::new()
		.width(Length::Fixed(size))
		.height(Length::Fixed(size))
		.circle_radius(3.0)
		.into()
}
