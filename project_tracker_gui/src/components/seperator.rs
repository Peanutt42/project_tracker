use iced::{Color, Length};
use iced_aw::{quad::Quad, widgets::InnerBounds};
use crate::styles::GREY;

pub fn horizontal_seperator() -> Quad {
	colored_horizontal_seperator(GREY)
}

pub fn colored_horizontal_seperator(color: Color) -> Quad {
	Quad {
		width: Length::Fill,
		height: Length::Fixed(1.0),
		inner_bounds: InnerBounds::Ratio(1.0, 1.0),
		quad_color: color.into(),
		..Default::default()
	}
}

pub fn vertical_seperator() -> Quad {
	Quad {
		width: Length::Fixed(1.0),
		height: Length::Fill,
		inner_bounds: InnerBounds::Ratio(1.0, 1.0),
		quad_color: GREY.into(),
		..Default::default()
	}
}