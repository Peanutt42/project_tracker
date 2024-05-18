use iced::{Element, Length, Background, Color};
use iced_aw::quad::Quad;
use crate::project_tracker::UiMessage;

pub fn horizontal_seperator(heigth: f32) -> Element<'static, UiMessage> {
	Quad {
		width: Length::Fill,
		height: Length::Fixed(heigth),
		inner_bounds: iced_aw::widgets::InnerBounds::Ratio(1.0, 1.0),
		quad_color: Background::Color(Color::from_rgb(0.5, 0.5, 0.5)),
		..Default::default()
	}
	.into()
}

pub fn vertical_seperator(width: f32) -> Element<'static, UiMessage> {
	Quad {
		width: Length::Fixed(width),
		height: Length::Fill,
		inner_bounds: iced_aw::widgets::InnerBounds::Ratio(1.0, 1.0),
		quad_color: Background::Color(Color::from_rgb(0.5, 0.5, 0.5)),
		..Default::default()
	}
	.into()
}