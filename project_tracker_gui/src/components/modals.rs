use iced::{alignment::Horizontal, theme, widget::{button, row, text}, Element, Length};
use iced_aw::{card, CardStyles};
use crate::{project_tracker::UiMessage, styles::{ConfirmModalCardStyle, DangerousButtonStyle, SPACING_AMOUNT}, components::copy_to_clipboard_button};

#[derive(Clone, Debug)]
pub enum ConfirmModalMessage {
	Open {
		title: &'static str,
		on_confirmed: Box<UiMessage>,
	},
	Close,
}

impl ConfirmModalMessage {
	pub fn open(title: &'static str, on_confirmed: impl Into<UiMessage>) -> UiMessage {
		Self::Open {
			title,
			on_confirmed: Box::new(on_confirmed.into()),
		}.into()
	}
}

impl From<ConfirmModalMessage> for UiMessage {
	fn from(value: ConfirmModalMessage) -> Self {
		UiMessage::ConfirmModalMessage(value)
	}
}

pub enum ConfirmModal {
	Opened {
		title: &'static str,
		on_confirmed: UiMessage,
	},
	Closed,
}

impl ConfirmModal {
	pub fn update(&mut self, message: ConfirmModalMessage) {
		match message {
			ConfirmModalMessage::Open { title, on_confirmed } => {
				*self = ConfirmModal::Opened { title, on_confirmed: *on_confirmed };
			},
			ConfirmModalMessage::Close => {
				*self = ConfirmModal::Closed;
			},
		}
	}

	pub fn view(&self) -> Option<Element<'static, UiMessage>> {
		match self {
			ConfirmModal::Closed => None,
			ConfirmModal::Opened { title, on_confirmed } => {
				Some(
					card(
						text(title),
						row![
							button(
								text("Ok")
									.horizontal_alignment(Horizontal::Center)
							)
							.width(Length::Fill)
							.on_press(UiMessage::ConfirmModalConfirmed(Box::new(on_confirmed.clone()))),

							button(
								text("Cancel")
									.horizontal_alignment(Horizontal::Center)
							)
							.width(Length::Fill)
							.style(theme::Button::Secondary)
							.on_press(ConfirmModalMessage::Close.into()),
						]
						.spacing(SPACING_AMOUNT)
					)
					.max_width(300.0)
					.style(CardStyles::custom(ConfirmModalCardStyle))
					.into()
				)
			},
		}
	}
}


pub enum ErrorMsgModal {
	Open {
		error_msg: String
	},
	Closed,
}

#[derive(Clone, Debug)]
pub enum ErrorMsgModalMessage {
	Open(String),
	Close,
}

impl ErrorMsgModalMessage {
	pub fn open(error_msg: String) -> UiMessage {
		Self::Open(error_msg).into()
	}
}

impl From<ErrorMsgModalMessage> for UiMessage {
	fn from(value: ErrorMsgModalMessage) -> Self {
		UiMessage::ErrorMsgModalMessage(value)
	}
}

impl ErrorMsgModal {
	pub fn update(&mut self, message: ErrorMsgModalMessage) {
		match message {
			ErrorMsgModalMessage::Open(error_msg) => {
				*self = ErrorMsgModal::Open { error_msg };
			},
			ErrorMsgModalMessage::Close => {
				*self = ErrorMsgModal::Closed;
			},
		}
	}

	pub fn view(&self) -> Option<Element<'static, UiMessage>> {
		match self {
			ErrorMsgModal::Open { error_msg } => {
				Some(
					card(
						text(error_msg),
						row![
							button(
								text("Ok")
									.horizontal_alignment(Horizontal::Center)
									.width(Length::Fill)
							)
							.width(Length::Fill)
							.style(theme::Button::custom(DangerousButtonStyle))
							.on_press(ErrorMsgModalMessage::Close.into()),

							copy_to_clipboard_button(error_msg.clone()),
						]
					)
					.max_width(300.0)
					.style(CardStyles::custom(ConfirmModalCardStyle))
					.into()
				)
			},
			ErrorMsgModal::Closed => None,
		}
	}
}