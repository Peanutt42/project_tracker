use iced::{Color, Length};
use iced_aw::quad::Quad;
use crate::styles::GREY;

pub fn horizontal_seperator() -> Quad {
	colored_horizontal_seperator(GREY)
}

pub fn colored_horizontal_seperator(color: Color) -> Quad {
	Quad {
		width: Length::Fill,
		height: Length::Fixed(1.0),
		inner_bounds: iced_aw::widgets::InnerBounds::Ratio(1.0, 1.0),
		quad_color: color.into(),
		..Default::default()
	}
}

pub fn partial_horizontal_seperator() -> Quad {
	Quad {
		width: Length::Fill,
		height: Length::Fixed(1.0),
		inner_bounds: iced_aw::widgets::InnerBounds::Ratio(0.8, 1.0),
		quad_color: GREY.into(),
		..Default::default()
	}
}
