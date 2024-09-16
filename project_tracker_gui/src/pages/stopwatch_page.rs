use std::time::{Duration, Instant};
use iced::{alignment::Horizontal, widget::{canvas, column, row, container, text}, window, Alignment, Element, Font, Length, Subscription};
use notify_rust::{Hint, Notification};
use crate::{components::{pause_timer_button, resume_timer_button, start_timer_button, stop_timer_button, StopwatchClock}, core::{Database, ProjectId, TaskId}, project_tracker::UiMessage, styles::LARGE_SPACING_AMOUNT};

#[derive(Debug, Default)]
pub enum StopwatchPage {
	#[default]
	Idle,
	Ticking {
		elapsed_time: Duration,
		last_update: Instant,
		paused: bool,
		task: Option<(ProjectId, TaskId)>,
		clock: StopwatchClock,
		finished_notification_sent: bool,
	},
}

#[derive(Clone, Debug)]
pub enum StopwatchPageMessage {
	Start {
		task: Option<(ProjectId, TaskId)>,
	},
	Stop,
	Pause,
	Resume,
	Toggle,
	RedrawClock,
}

impl From<StopwatchPageMessage> for UiMessage {
	fn from(value: StopwatchPageMessage) -> Self {
		UiMessage::StopwatchPageMessage(value)
	}
}

impl StopwatchPage {
	pub fn clock(&self) -> Option<&StopwatchClock> {
		match self {
			StopwatchPage::Ticking { clock, .. } => Some(clock),
			StopwatchPage::Idle => None,
		}
	}

	pub fn subscription(&self) -> Subscription<UiMessage> {
		match self {
			StopwatchPage::Idle => Subscription::none(),
			StopwatchPage::Ticking { .. } => window::frames().map(|_| StopwatchPageMessage::RedrawClock.into()),
		}
	}

	pub fn update(&mut self, message: StopwatchPageMessage, database: &Option<Database>) {
		match message {
			StopwatchPageMessage::Start{ task } => {
				*self = StopwatchPage::Ticking {
					elapsed_time: Duration::ZERO,
					last_update: Instant::now(),
					paused: false,
					task,
					clock: StopwatchClock::new(0.0, String::new(), String::new()),
					finished_notification_sent: false
				};
			},
			StopwatchPageMessage::Stop => {
				*self = StopwatchPage::Idle;
			},
			StopwatchPageMessage::Resume => {
				if let StopwatchPage::Ticking { paused, .. } = self {
					*paused = false;
				}
			},
			StopwatchPageMessage::Pause => {
				if let StopwatchPage::Ticking { paused, .. } = self {
					*paused = true;
				}
			},
			StopwatchPageMessage::Toggle => {
				match self {
					StopwatchPage::Idle => self.update(StopwatchPageMessage::Start{ task: None }, database),
					StopwatchPage::Ticking { .. } => self.update(StopwatchPageMessage::Stop, database),
				}
			},
			StopwatchPageMessage::RedrawClock => {
				if let StopwatchPage::Ticking { clock, task, elapsed_time, finished_notification_sent, paused, last_update } = self {
					if !*paused {
						*elapsed_time += Instant::now().duration_since(*last_update);
					}
					*last_update = Instant::now();

					let task = task.as_ref().and_then(|(project_id, task_id)|
						database.as_ref().and_then(|db|
							db.projects()
								.get(project_id)
								.and_then(|project| project.get_task(task_id))
						)
					);

					if let Some(task) = task {
						if let Some(needed_minutes) = task.needed_time_minutes {
							let timer_seconds = elapsed_time.as_secs_f32();
							let needed_seconds = needed_minutes as f32 * 60.0;
							let seconds_left = needed_seconds - timer_seconds;
							clock.set_percentage(timer_seconds / needed_seconds);
							clock.set_label(format_stopwatch_duration(seconds_left.round_ties_even() as i64));
							clock.set_sub_label(format_stopwatch_duration(needed_seconds.round_ties_even() as i64));

							if seconds_left <= 0.0 && !*finished_notification_sent {
								let summary = format!("{} min. timer finished!", needed_minutes);
								let body = &task.name;

								if cfg!(target_os = "linux") {
									let _ = Notification::new()
										.summary(&summary)
										.body(body)
										.appname("Project Tracker")
										.icon("Project Tracker")
										.hint(Hint::DesktopEntry("Project Tracker".to_string()))
										.show();
								}
								else {
									let _ = Notification::new()
	            						.summary(&summary)
										.body(body)
										.show();
								}

								*finished_notification_sent = true;
							}
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
				StopwatchPage::Ticking { elapsed_time, task, clock, paused, .. } => {
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
						text(format_stopwatch_duration(elapsed_time.as_secs_f64().round_ties_even() as i64))
							.font(Font::DEFAULT)
							.size(90)
							.width(Length::Fill)
							.horizontal_alignment(Horizontal::Center)
							.into()
					};

					column![
						clock,

						row![
							if *paused {
								resume_timer_button()
							}
							else {
								pause_timer_button()
							},
							stop_timer_button()
						]
						.spacing(LARGE_SPACING_AMOUNT)
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