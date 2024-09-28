use std::time::{Duration, Instant};
use iced::{alignment::{Horizontal, Vertical}, padding::{left, top}, widget::{canvas, column, container, row, text, Column, Row}, window, Alignment, Element, Font, Length::{self, Fill}, Subscription};
use notify_rust::Notification;
use crate::{components::{complete_task_timer_button, days_left_widget, pause_timer_button, project_color_block, resume_timer_button, start_timer_button, stop_timer_button, StopwatchClock}, core::{Database, DatabaseMessage, PreferenceMessage, ProjectId, StopwatchProgress, TaskId}, project_tracker::UiMessage, styles::{task_tag_container_style, LARGE_PADDING_AMOUNT, LARGE_SPACING_AMOUNT, LARGE_TEXT_SIZE, PADDING_AMOUNT, SMALL_PADDING_AMOUNT, SPACING_AMOUNT, TINY_SPACING_AMOUNT}};

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
	StartupAgain {
		task: Option<(ProjectId, TaskId)>,
		elapsed_time: Duration,
		paused: bool,
		finished_notification_sent: bool,
	},
	Stop,
	Pause,
	Resume,
	Toggle,
	CompleteTask,
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

	fn get_progress(&self) -> Option<StopwatchProgress> {
		match self {
			StopwatchPage::Idle => None,
			StopwatchPage::Ticking { elapsed_time, paused, task, finished_notification_sent, .. } => {
				Some(
					StopwatchProgress {
						task: *task,
						elapsed_time_seconds: elapsed_time.as_secs(),
						paused: *paused,
						finished_notification_sent: *finished_notification_sent,
					}
				)
			},
		}
	}

	pub fn update(&mut self, message: StopwatchPageMessage, database: &Option<Database>, opened: bool) -> Option<Vec<UiMessage>> {
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
				Some(vec![
					UiMessage::OpenStopwatch,
					PreferenceMessage::SetStopwatchProgress(self.get_progress()).into()
				])
			},
			StopwatchPageMessage::StartupAgain { task, elapsed_time, paused, finished_notification_sent } => {
				*self = StopwatchPage::Ticking {
					elapsed_time,
					last_update: Instant::now(),
					paused,
					task,
					clock: StopwatchClock::new(0.0, String::new(), String::new()),
					finished_notification_sent
				};
				Some(vec![
					UiMessage::OpenStopwatch
				])
			},
			StopwatchPageMessage::Stop => {
				*self = StopwatchPage::Idle;
				Some(vec![PreferenceMessage::SetStopwatchProgress(self.get_progress()).into()])
			},
			StopwatchPageMessage::Resume => {
				if let StopwatchPage::Ticking { paused, .. } = self {
					*paused = false;
					Some(vec![PreferenceMessage::SetStopwatchProgress(self.get_progress()).into()])
				}
				else {
					None
				}
			},
			StopwatchPageMessage::Pause => {
				if let StopwatchPage::Ticking { paused, .. } = self {
					*paused = true;
					Some(vec![PreferenceMessage::SetStopwatchProgress(self.get_progress()).into()])
				}
				else {
					None
				}
			},
			StopwatchPageMessage::Toggle => {
				if opened {
					match self {
						StopwatchPage::Idle => self.update(StopwatchPageMessage::Start{ task: None }, database, opened),
						StopwatchPage::Ticking { paused, .. } => {
							if *paused {
								self.update(StopwatchPageMessage::Resume, database, opened);
							}
							else {
								self.update(StopwatchPageMessage::Pause, database, opened);
							}
							None
						},
					}
				}
				else {
					None
				}
			},
			StopwatchPageMessage::CompleteTask => {
				let database_action: Option<UiMessage> = if let StopwatchPage::Ticking { task: Some((project_id, task_id)), .. } = self {
					Some(DatabaseMessage::SetTaskDone { project_id: *project_id, task_id: *task_id }.into())
				}
				else {
					None
				};
				let actions = self.update(StopwatchPageMessage::Stop, database, opened);

				if let Some(mut actions) = actions {
					if let Some(database_action) = database_action {
						actions.push(database_action);
					}
					Some(actions)
				}
				else {
					database_action.map(|db_action| vec![db_action])
				}
			},
			StopwatchPageMessage::RedrawClock => {
				if let StopwatchPage::Ticking { clock, task, elapsed_time, finished_notification_sent, paused, last_update } = self {
					if !*paused {
						*elapsed_time += Instant::now().duration_since(*last_update);
					}
					*last_update = Instant::now();

					let task_ref = task.as_ref().and_then(|(project_id, task_id)|
						database.as_ref().and_then(|db|
							db.get_task(project_id, task_id)
						)
					);

					if let Some(task) = task_ref {
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

								#[cfg(target_os = "linux")]
								{
									use notify_rust::Hint;

									let _ = Notification::new()
										.summary(&summary)
										.body(body)
										.appname("Project Tracker")
										.icon("Project Tracker")
										.hint(Hint::DesktopEntry("Project Tracker".to_string()))
										.show();
								}

								#[cfg(not(target_os = "linux"))]
								{
									let _ = Notification::new()
	            						.summary(&summary)
										.body(body)
										.show();
								}

								*finished_notification_sent = true;
							}
						}
					}
					Some(vec![
						PreferenceMessage::SetStopwatchProgress(self.get_progress()).into(),
					])
				}
				else {
					None
				}
			},
		}
	}

	pub fn view<'a>(&'a self, database: &'a Option<Database>) -> Element<'a, UiMessage> {
		container(
			match self {
				StopwatchPage::Idle => {
					column![
						text("Start any task!").size(90),

						start_timer_button()
					]
					.align_x(Alignment::Center)
					.spacing(LARGE_SPACING_AMOUNT)
				},
				StopwatchPage::Ticking { elapsed_time, task, clock, paused, .. } => {
					let mut project_ref = None;
					let mut task_ref = None;
					if let Some((project_id, task_id)) = &task {
						if let Some(database) = database {
							if let Some(project) = database.get_project(project_id) {
								project_ref = Some(project);
								task_ref = project.get_task(task_id);
							}
						}
					}

					let clock: Element<UiMessage> = if task_ref.is_some() {
						canvas(clock)
							.width(Length::Fixed(300.0))
							.height(Length::Fixed(300.0))
							.into()
					}
					else {
						text(format_stopwatch_duration(elapsed_time.as_secs_f64().round_ties_even() as i64))
							.font(Font::DEFAULT)
							.size(90)
							.width(Fill)
							.align_x(Horizontal::Center)
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
						.push_maybe(task.map(|_| {
							complete_task_timer_button()
						}))
						.spacing(LARGE_SPACING_AMOUNT)
					]
					.push_maybe(
						task_ref.map(|task| {
							Column::new()
								.push_maybe(
									project_ref.map(|project| {
										row![
											project_color_block(project.color.into()),
											text(format!("{}:", project.name)).size(LARGE_TEXT_SIZE),
											Row::with_children(
												task.tags.iter().map(|tag_id| {
													if let Some(tag) = project.task_tags.get(tag_id) {
														container(text(&tag.name))
	                										.style(|t| task_tag_container_style(t, tag.color.into()))
	                          								.padding(SMALL_PADDING_AMOUNT)
															.into()
													}
													else {
														"<invalid tag id>".into()
													}
												})
											)
											.spacing(SPACING_AMOUNT)
											.padding(left(PADDING_AMOUNT))
										]
										.spacing(TINY_SPACING_AMOUNT)
										.align_y(Vertical::Center)
									})
								)
								.push(text(&task.name).size(LARGE_TEXT_SIZE))
								.push_maybe(task.due_date.map(|due_date| {
									days_left_widget(due_date)
								}))
								.spacing(SPACING_AMOUNT)
								.padding(top(LARGE_PADDING_AMOUNT))
						})
					)
					.align_x(Alignment::Center)
					.spacing(LARGE_SPACING_AMOUNT)
				},
			}
		)
		.center_x(Fill)
		.center_y(Fill)
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