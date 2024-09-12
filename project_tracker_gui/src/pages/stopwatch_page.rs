use std::time::{Duration, Instant};
use iced::{alignment::Horizontal, time, widget::{column, container, text}, Alignment, Element, Font, Length, Subscription};
use crate::{components::{start_timer_button, stop_timer_button}, project_tracker::UiMessage, styles::LARGE_SPACING_AMOUNT};

#[derive(Clone, Debug, Default)]
pub enum StopwatchPage {
	#[default]
	Idle,
	Ticking {
		timer_start: Instant,
	}
}

#[derive(Clone, Debug)]
pub enum StopwatchPageMessage {
	Start,
	Stop,
	Toggle,
	RedrawTimer,
}

impl From<StopwatchPageMessage> for UiMessage {
	fn from(value: StopwatchPageMessage) -> Self {
		UiMessage::StopwatchPageMessage(value)
	}
}

impl StopwatchPage {
	pub fn subscription(&self) -> Subscription<UiMessage> {
		match self {
			StopwatchPage::Idle => Subscription::none(),
			StopwatchPage::Ticking { .. } => time::every(Duration::from_secs(1)).map(|_| StopwatchPageMessage::RedrawTimer.into()),
		}
	}

	pub fn update(&mut self, message: StopwatchPageMessage) {
		match message {
			StopwatchPageMessage::Start => {
				*self = StopwatchPage::Ticking { timer_start: Instant::now() };
			},
			StopwatchPageMessage::Stop => {
				*self = StopwatchPage::Idle;
			},
			StopwatchPageMessage::Toggle => {
				match self {
					StopwatchPage::Idle => self.update(StopwatchPageMessage::Start),
					StopwatchPage::Ticking { .. } => self.update(StopwatchPageMessage::Stop),
				}
			},
			StopwatchPageMessage::RedrawTimer => {}, // makes iced redraw the stopwatch timer
		}
	}

	pub fn view(&self) -> Element<UiMessage> {
		container(
			match self {
				StopwatchPage::Idle => {
					column![
						text("Start task!").size(90),

						start_timer_button()
					]
					.align_items(Alignment::Center)
					.spacing(LARGE_SPACING_AMOUNT)
				},
				StopwatchPage::Ticking { timer_start } => {
					column![
						text(format_stopwatch_duration(*timer_start))
							.font(Font::DEFAULT)
							.size(90)
							.width(Length::Fill)
							.horizontal_alignment(Horizontal::Center),

						stop_timer_button()
					]
					.align_items(Alignment::Center)
					.spacing(LARGE_SPACING_AMOUNT)
				},
			}
		)
		.width(Length::Fill)
		.height(Length::Fill)
		.center_x()
		.center_y()
		.into()
	}
}

pub fn format_stopwatch_duration(timer_start: Instant) -> String {
	const MINUTE: u64 = 60;
	const HOUR: u64 = 60 * MINUTE;

	let total_seconds = Instant::now().duration_since(timer_start).as_secs();
	let hours = total_seconds / HOUR;
	let minutes = (total_seconds % HOUR) / MINUTE;
	let seconds = total_seconds % MINUTE;

	if hours > 0 {
		format!("{hours:0>2}:{minutes:0>2}:{seconds:0>2}")
	}
	else {
		format!("{minutes:0>2}:{seconds:0>2}")
	}
}