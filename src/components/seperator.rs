use iced::{Element, Length, Background, Color};
use iced_aw::quad::Quad;
use crate::project_tracker::UiMessage;



pub fn horizontal_seperator(heigth: f32) -> Element<'static, UiMessage> {
	Quad {
		width: Length::Fill,
		height: Length::Fixed(heigth),
		inner_bounds: iced_aw::widgets::InnerBounds::Ratio(0.8, 1.0), // 0.8 leaves some space left and right of the seperator (10% on each side)
		quad_color: Background::Color(Color::from_rgb(0.5, 0.5, 0.5)),
		..Default::default()
	}
	.into()
}