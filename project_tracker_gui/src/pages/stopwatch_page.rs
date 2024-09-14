use std::time::Instant;
use iced::{alignment::Horizontal, widget::{canvas, column, container, text}, window, Alignment, Element, Font, Length, Subscription};
use crate::{components::{start_timer_button, stop_timer_button, StopwatchClock}, core::{Database, ProjectId, TaskId}, project_tracker::UiMessage, styles::LARGE_SPACING_AMOUNT};

#[derive(Debug, Default)]
pub enum StopwatchPage {
	#[default]
	Idle,
	Ticking {
		timer_start: Instant,
		task: Option<(ProjectId, TaskId)>,
		clock: StopwatchClock,
	}
}

#[derive(Clone, Debug)]
pub enum StopwatchPageMessage {
	Start {
		task: Option<(ProjectId, TaskId)>,
	},
	Stop,
	Toggle,
	RedrawClock,
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
			StopwatchPage::Ticking { .. } => window::frames().map(|_| StopwatchPageMessage::RedrawClock.into()),
		}
	}

	pub fn update(&mut self, message: StopwatchPageMessage, database: &Option<Database>) {
		match message {
			StopwatchPageMessage::Start{ task } => {
				*self = StopwatchPage::Ticking { timer_start: Instant::now(), task, clock: StopwatchClock::new(0.0, String::new(), String::new()) };
			},
			StopwatchPageMessage::Stop => {
				*self = StopwatchPage::Idle;
			},
			StopwatchPageMessage::Toggle => {
				match self {
					StopwatchPage::Idle => self.update(StopwatchPageMessage::Start{ task: None }, database),
					StopwatchPage::Ticking { .. } => self.update(StopwatchPageMessage::Stop, database),
				}
			},
			StopwatchPageMessage::RedrawClock => {
				if let StopwatchPage::Ticking { clock, task, timer_start } = self {
					let task = task.as_ref().and_then(|(project_id, task_id)|
						database.as_ref().and_then(|db|
							db.projects()
								.get(project_id)
								.and_then(|project| project.get_task(task_id))
						)
					);

					if let Some(task) = task {
						if let Some(needed_minutes) = task.needed_time_minutes {
							let timer_seconds = Instant::now().duration_since(*timer_start).as_secs_f32();
							let needed_seconds = needed_minutes as f32 * 60.0;
							let seconds_left = needed_seconds - timer_seconds;
							clock.set_percentage(timer_seconds / needed_seconds);
							clock.set_label(format_stopwatch_duration(seconds_left.round_ties_even() as i64));
							clock.set_sub_label(format_stopwatch_duration(needed_seconds.round_ties_even() as i64));
						}
					}
				}
			},
		}
	}

	pub fn view(&self, database: &Option<Database>) -> Element<UiMessage> {
		container(
			match self {
				StopwatchPage::Idle => {
					column![
						text("Start any task!").size(90),

						start_timer_button()
					]
					.align_items(Alignment::Center)
					.spacing(LARGE_SPACING_AMOUNT)
				},
				StopwatchPage::Ticking { timer_start, task, clock } => {
					let task = task.as_ref().and_then(|(project_id, task_id)|
						database.as_ref().and_then(|db|
							db.projects()
								.get(project_id)
								.and_then(|project| project.get_task(task_id))
						)
					);

					let clock: Element<UiMessage> = if task.is_some() {
						canvas(clock)
							.width(Length::Fixed(300.0))
							.height(Length::Fixed(300.0))
							.into()
					}
					else {
						text(format_stopwatch_duration(Instant::now().duration_since(*timer_start).as_secs_f64().round_ties_even() as i64))
							.font(Font::DEFAULT)
							.size(90)
							.width(Length::Fill)
							.horizontal_alignment(Horizontal::Center)
							.into()
					};

					column![
						clock,

						stop_timer_button()
					]
					.push_maybe(
						task.map(|task| text(&task.name))
					)
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

pub fn format_stopwatch_duration(total_seconds: i64) -> String {
	const MINUTE: i64 = 60;
	const HOUR: i64 = 60 * MINUTE;

	let hours = total_seconds.abs() / HOUR;
	let minutes = (total_seconds.abs() % HOUR) / MINUTE;
	let seconds = total_seconds.abs() % MINUTE;

	let factor_str = if total_seconds >= 0 { "" } else { "-" };

	if hours > 0 {
		format!("{factor_str}{hours:0>2}:{minutes:0>2}:{seconds:0>2}")
	}
	else {
		format!("{factor_str}{minutes:0>2}:{seconds:0>2}")
	}
}