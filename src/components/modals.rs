use crate::{
	components::{confirm_cancel_button, confirm_ok_button, copy_to_clipboard_button},
	project_tracker::Message,
	styles::{card_style, dangerous_button_style, SPACING_AMOUNT},
};
use iced::{
	alignment::Horizontal,
	widget::{button, row, text},
	Element,
	Length::Fill,
};
use iced_aw::card;


