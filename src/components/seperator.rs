use iced::{Length, Background, Color};
use iced_aw::quad::Quad;

pub fn horizontal_seperator(heigth: f32) -> Quad {
	Quad {
		width: Length::Fill,
		height: Length::Fixed(heigth),
		inner_bounds: iced_aw::widgets::InnerBounds::Ratio(1.0, 1.0),
		quad_color: Background::Color(Color::from_rgb(0.5, 0.5, 0.5)),
		..Default::default()
	}
}

pub fn partial_horizontal_seperator(heigth: f32) -> Quad {
	Quad {
		width: Length::Fill,
		height: Length::Fixed(heigth),
		inner_bounds: iced_aw::widgets::InnerBounds::Ratio(0.8, 1.0),
		quad_color: Background::Color(Color::from_rgb(0.5, 0.5, 0.5)),
		..Default::default()
	}
}