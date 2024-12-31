use crate::{
	components::{confirm_cancel_button, confirm_ok_button},
	project_tracker::Message,
	styles::{card_style, SPACING_AMOUNT},
};
use iced::{
	widget::{row, text},
	Element,
};
use iced_aw::card;

#[derive(Clone, Debug)]
pub enum ConfirmModalMessage {
	Open {
		title: String,
		on_confirmed: Box<Message>,
		custom_ok_label: Option<&'static str>,
		custom_cancel_label: Option<&'static str>,
	},
	Close,
}

impl ConfirmModalMessage {
	pub fn open(title: String, on_confirmed: impl Into<Message>) -> Message {
		Self::Open {
			title,
			on_confirmed: Box::new(on_confirmed.into()),
			custom_ok_label: None,
			custom_cancel_label: None,
		}
		.into()
	}
}

impl From<ConfirmModalMessage> for Message {
	fn from(value: ConfirmModalMessage) -> Self {
		Message::ConfirmModalMessage(value)
	}
}

pub struct ConfirmModal {
	title: String,
	pub on_confirmed: Message,
	custom_ok_label: Option<&'static str>,
	custom_cancel_label: Option<&'static str>,
}

impl ConfirmModal {
	pub fn new(
		title: String,
		on_confirmed: Message,
		custom_ok_label: Option<&'static str>,
		custom_cancel_label: Option<&'static str>,
	) -> Self {
		Self {
			title,
			on_confirmed,
			custom_ok_label,
			custom_cancel_label,
		}
	}

	pub fn view(&self) -> Element<Message> {
		card(
			text(&self.title),
			row![
				confirm_ok_button(&self.on_confirmed, self.custom_ok_label),
				confirm_cancel_button(self.custom_cancel_label)
			]
			.spacing(SPACING_AMOUNT),
		)
		.max_width(300.0)
		.style(card_style)
		.into()
	}
}
