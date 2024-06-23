use iced::{alignment::Horizontal, widget::{button, row, text}, Element, Length, theme};
use iced_aw::{card, CardStyles};
use crate::{project_tracker::UiMessage, styles::{ConfirmModalCardStyle, SPACING_AMOUNT}};

#[derive(Clone, Debug)]
pub enum ConfirmModalMessage {
	Open {
		title: String,
		on_confirmed: Box<UiMessage>,
	},
	Close,
}

impl From<ConfirmModalMessage> for UiMessage {
	fn from(value: ConfirmModalMessage) -> Self {
		UiMessage::ConfirmModalMessage(value)
	}
}

pub enum ConfirmModal {
	Opened {
		title: String,
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