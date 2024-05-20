use iced::Padding;


pub const SMALL_PADDING_AMOUNT: f32 = 5.0;
pub const PADDING_AMOUNT: f32 = 10.0;
pub const LARGE_PADDING_AMOUNT: f32 = 20.0;

pub const HORIZONTAL_PADDING: Padding = Padding {
	left: PADDING_AMOUNT,
	right: PADDING_AMOUNT,
	..Padding::ZERO
};

pub const SMALL_HORIZONTAL_PADDING: Padding = Padding {
	left: SMALL_PADDING_AMOUNT,
	right: SMALL_PADDING_AMOUNT,
	..Padding::ZERO
};