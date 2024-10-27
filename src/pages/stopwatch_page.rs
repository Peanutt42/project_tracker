use crate::{
	components::{
		complete_task_timer_button, days_left_widget, horizontal_scrollable, pause_timer_button, resume_timer_button, start_timer_button, stop_timer_button, task_description, StopwatchClock, HORIZONTAL_SCROLLABLE_PADDING
	},
	core::{Database, DatabaseMessage, PreferenceMessage, Project, ProjectId, StopwatchProgress, Task, TaskId, TaskType},
	project_tracker::Message,
	styles::{
		task_tag_container_style, BOLD_FONT, HEADING_TEXT_SIZE, LARGE_PADDING_AMOUNT, LARGE_SPACING_AMOUNT, PADDING_AMOUNT, SMALL_PADDING_AMOUNT, SPACING_AMOUNT
	}, ProjectTrackerApp,
};
use iced::{
	advanced::graphics::futures::backend::default::time, alignment::{Horizontal, Vertical}, keyboard, padding::top, widget::{canvas, column, container, responsive, row, text, Column, Row}, window, Alignment, Element, Font, Length::{self, Fill}, Padding, Subscription
};
use notify_rust::Notification;
use std::time::{Duration, Instant};

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
	UpdateClock,
}

impl From<StopwatchPageMessage> for Message {
	fn from(value: StopwatchPageMessage) -> Self {
		Message::StopwatchPageMessage(value)
	}
}

impl StopwatchPage {
	pub fn clock(&self) -> Option<&StopwatchClock> {
		match self {
			StopwatchPage::Ticking { clock, .. } => Some(clock),
			StopwatchPage::Idle => None,
		}
	}

	pub fn subscription(&self, opened: bool) -> Subscription<StopwatchPageMessage> {
		let redraw_subscription = match self {
			StopwatchPage::Idle => Subscription::none(),
			StopwatchPage::Ticking { .. } => {
				if opened {
					window::frames().map(|_| StopwatchPageMessage::UpdateClock)
				} else {
					time::every(Duration::from_secs(1)).map(|_| StopwatchPageMessage::UpdateClock)
				}
			}
		};

		let toggle_subscription = keyboard::on_key_press(|key, modifiers| match key.as_ref() {
			keyboard::Key::Named(keyboard::key::Named::Space)
				if !modifiers.command() && !modifiers.shift() =>
			{
				Some(StopwatchPageMessage::Toggle)
			}
			_ => None,
		});

		Subscription::batch([redraw_subscription, toggle_subscription])
	}

	fn get_progress(&self) -> Option<StopwatchProgress> {
		match self {
			StopwatchPage::Idle => None,
			StopwatchPage::Ticking {
				elapsed_time,
				paused,
				task,
				finished_notification_sent,
				..
			} => Some(StopwatchProgress {
				task: *task,
				elapsed_time_seconds: elapsed_time.as_secs(),
				paused: *paused,
				finished_notification_sent: *finished_notification_sent,
			}),
		}
	}

	pub fn update(
		&mut self,
		message: StopwatchPageMessage,
		database: &Option<Database>,
		opened: bool,
	) -> Option<Vec<Message>> {
		match message {
			StopwatchPageMessage::Start { task } => {
				*self = StopwatchPage::Ticking {
					elapsed_time: Duration::ZERO,
					last_update: Instant::now(),
					paused: false,
					task,
					clock: StopwatchClock::new(0.0, String::new(), String::new()),
					finished_notification_sent: false,
				};
				Some(vec![
					Message::OpenStopwatch,
					PreferenceMessage::SetStopwatchProgress(self.get_progress()).into(),
				])
			}
			StopwatchPageMessage::StartupAgain {
				task,
				elapsed_time,
				paused,
				finished_notification_sent,
			} => {
				*self = StopwatchPage::Ticking {
					elapsed_time,
					last_update: Instant::now(),
					paused,
					task,
					clock: StopwatchClock::new(0.0, String::new(), String::new()),
					finished_notification_sent,
				};
				Some(vec![
					StopwatchPageMessage::UpdateClock.into(),
					Message::OpenStopwatch
				])
			}
			StopwatchPageMessage::Stop => {
				*self = StopwatchPage::Idle;
				Some(vec![PreferenceMessage::SetStopwatchProgress(
					self.get_progress(),
				)
				.into()])
			}
			StopwatchPageMessage::Resume => {
				if let StopwatchPage::Ticking { paused, .. } = self {
					*paused = false;
					Some(vec![PreferenceMessage::SetStopwatchProgress(
						self.get_progress(),
					)
					.into()])
				} else {
					None
				}
			}
			StopwatchPageMessage::Pause => {
				if let StopwatchPage::Ticking { paused, .. } = self {
					*paused = true;
					Some(vec![PreferenceMessage::SetStopwatchProgress(
						self.get_progress(),
					)
					.into()])
				} else {
					None
				}
			}
			StopwatchPageMessage::Toggle => {
				if opened {
					match self {
						StopwatchPage::Idle => self.update(
							StopwatchPageMessage::Start { task: None },
							database,
							opened,
						),
						StopwatchPage::Ticking { paused, .. } => {
							if *paused {
								self.update(StopwatchPageMessage::Resume, database, opened);
							} else {
								self.update(StopwatchPageMessage::Pause, database, opened);
							}
							None
						}
					}
				} else {
					None
				}
			}
			StopwatchPageMessage::CompleteTask => {
				let database_action: Option<Message> = if let StopwatchPage::Ticking {
					task: Some((project_id, task_id)),
					..
				} = self
				{
					Some(
						DatabaseMessage::SetTaskDone {
							project_id: *project_id,
							task_id: *task_id,
						}
						.into(),
					)
				} else {
					None
				};
				let actions = self.update(StopwatchPageMessage::Stop, database, opened);

				if let Some(mut actions) = actions {
					if let Some(database_action) = database_action {
						actions.push(database_action);
					}
					Some(actions)
				} else {
					database_action.map(|db_action| vec![db_action])
				}
			}
			StopwatchPageMessage::UpdateClock => {
				if let StopwatchPage::Ticking {
					clock,
					task,
					elapsed_time,
					finished_notification_sent,
					paused,
					last_update,
				} = self
				{
					if !*paused {
						*elapsed_time += Instant::now().duration_since(*last_update);
					}
					*last_update = Instant::now();

					let task_and_type = task.as_ref().and_then(|(project_id, task_id)| {
						database
							.as_ref()
							.and_then(|db| db.get_task_and_type(project_id, task_id))
					});

					if let Some((task, task_type)) = task_and_type {
						if matches!(task_type, TaskType::Done) {
							*self = StopwatchPage::Idle;
						}
						else if let Some(needed_minutes) = task.needed_time_minutes {
							let timer_seconds = elapsed_time.as_secs_f32();
							let needed_seconds = needed_minutes as f32 * 60.0;
							let seconds_left = needed_seconds - timer_seconds;
							clock.set_percentage(timer_seconds / needed_seconds);
							clock.set_label(format_stopwatch_duration(
								seconds_left.round_ties_even() as i64,
							));
							clock.set_sub_label(format_stopwatch_duration(
								needed_seconds.round_ties_even() as i64,
							));

							if seconds_left <= 0.0 && !*finished_notification_sent {
								let summary = format!("{} min. timer finished!", needed_minutes);
								let body = task.name();

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
									let _ = Notification::new().summary(&summary).body(body).show();
								}

								*finished_notification_sent = true;
							}
						}
					}
					else {
						*self = StopwatchPage::Idle;
					}

					Some(vec![PreferenceMessage::SetStopwatchProgress(
						self.get_progress(),
					)
					.into()])
				} else {
					None
				}
			}
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<'a, Message> {
		container(match self {
			StopwatchPage::Idle => Element::new(column![
				text("Start any task!")
					.size(90),
				start_timer_button()
			]
			.align_x(Alignment::Center)
			.spacing(LARGE_SPACING_AMOUNT)),

			StopwatchPage::Ticking {
				elapsed_time,
				task,
				clock,
				paused,
				..
			} => {
				let mut project_ref = None;
				let mut task_ref = None;
				if let Some((project_id, task_id)) = &task {
					if let Some(database) = &app.database {
						if let Some(project) = database.get_project(project_id) {
							project_ref = Some(project);
							task_ref = project.get_task(task_id);
						}
					}
				}

				responsive(move |size| -> Element<Message> {
					let clock: Element<Message> = if task_ref.is_some() {
						canvas(clock)
							.width(Length::Fixed(300.0))
							.height(Length::Fixed(300.0))
							.into()
					} else {
						text(format_stopwatch_duration(
							elapsed_time.as_secs_f64().round_ties_even() as i64,
						))
						.font(Font::DEFAULT)
						.size(90)
						.width(Fill)
						.align_x(Horizontal::Center)
						.into()
					};

					let clock_side = column![
						clock,
						row![
							if *paused {
								resume_timer_button()
							} else {
								pause_timer_button()
							},
							stop_timer_button()
						]
						.push_maybe(task.map(|_| { complete_task_timer_button() }))
						.spacing(LARGE_SPACING_AMOUNT)
					]
					.align_x(Alignment::Center)
					.spacing(LARGE_SPACING_AMOUNT)
					.width(Fill);

					let page_view: Element<Message> = if size.width > size.height {
						row![
							clock_side,
						]
						.push_maybe(task_info(task_ref, project_ref, app))
						.spacing(LARGE_SPACING_AMOUNT)
						.into()
					}
					else {
						column![
							clock_side,
						]
						.push_maybe(task_info(task_ref, project_ref, app))
						.spacing(LARGE_SPACING_AMOUNT)
						.into()
					};

					container(
						page_view
					)
					.center(Fill)
					.into()
				})
				.into()
			}
		})
		.center(Fill)
		.padding(LARGE_PADDING_AMOUNT)
		.into()
	}
}

fn task_info<'a>(task: Option<&'a Task>, project: Option<&'a Project>, app: &'a ProjectTrackerApp) -> Option<Element<'a, Message>> {
	task.map(|task| {
		Column::new()
			.push(
				text(task.name())
					.size(HEADING_TEXT_SIZE)
					.font(BOLD_FONT)
			)
			.push_maybe(task.due_date.map(|date| days_left_widget(date, false)))
			.push(task_description(task, app))
			.push_maybe(project.map(|project| {
				row![
					container(
						container(
							text(format!("{}:", project.name))
						)
						.style(|t| {
							task_tag_container_style(t, project.color.into())
						})
						.padding(
							Padding::new(SMALL_PADDING_AMOUNT)
								.left(PADDING_AMOUNT)
								.right(PADDING_AMOUNT)
						)
					)
					.padding(HORIZONTAL_SCROLLABLE_PADDING),

					horizontal_scrollable(
						Row::with_children(task.tags.iter().map(|tag_id| {
							if let Some(tag) = project.task_tags.get(tag_id) {
								container(text(&tag.name))
									.style(|t| {
										task_tag_container_style(t, tag.color.into())
									})
									.padding(
										Padding::new(SMALL_PADDING_AMOUNT)
											.left(PADDING_AMOUNT)
											.right(PADDING_AMOUNT)
									)
									.into()
							} else {
								"<invalid tag id>".into()
							}
						}))
						.spacing(SPACING_AMOUNT)
					),
				]
				.spacing(SPACING_AMOUNT)
				.align_y(Vertical::Center)
			}))
			.spacing(LARGE_SPACING_AMOUNT)
			.padding(top(LARGE_PADDING_AMOUNT))
			.align_x(Horizontal::Center)
			.into()
	})
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
	} else {
		format!("{factor_str}{minutes:0>2}:{seconds:0>2}")
	}
}
