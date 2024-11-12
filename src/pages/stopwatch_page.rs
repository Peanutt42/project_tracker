use crate::{
	components::{
		complete_task_timer_button, days_left_widget, horizontal_scrollable, pause_timer_button, resume_timer_button, stop_timer_button, take_break_button, task_description, track_time_button, StopwatchClock, HORIZONTAL_SCROLLABLE_PADDING
	},
	core::{Database, DatabaseMessage, OptionalPreference, PreferenceMessage, Preferences, Project, ProjectId, StopwatchProgress, Task, TaskId, TaskType},
	project_tracker::Message,
	styles::{
		task_tag_container_style, BOLD_FONT, HEADING_TEXT_SIZE, LARGE_PADDING_AMOUNT, LARGE_SPACING_AMOUNT, PADDING_AMOUNT, SMALL_PADDING_AMOUNT, SPACING_AMOUNT
	}, ProjectTrackerApp,
};
use iced::{
	alignment::{Horizontal, Vertical}, keyboard, time, widget::{canvas, column, container, responsive, row, text, Column, Row, Space}, window, Alignment, Element, Font, Length::{self, Fill}, Padding, Subscription
};
use notify_rust::Notification;
use std::{io::Cursor, thread, time::{Duration, Instant}};

#[derive(Debug, Default)]
pub enum StopwatchPage {
	#[default]
	Idle,
	TrackTime {
		elapsed_time: Duration,
		last_update: Instant,
		paused: bool,
	},
	StopTaskTime {
		elapsed_time: Duration,
		last_update: Instant,
		paused: bool,
		project_id: ProjectId,
		task_id: TaskId,
		clock: StopwatchClock,
		finished_notification_sent: bool,
	},
	TakingBreak {
		elapsed_time: Duration,
		last_update: Instant,
		paused: bool,
		break_duration_minutes: usize,
		clock: StopwatchClock,
		break_over_notification_sent: bool,
	}
}

#[derive(Clone, Debug)]
pub enum StopwatchPageMessage {
	StartTrackingTime,
	StopTask {
		project_id: ProjectId,
		task_id: TaskId,
	},
	TakeBreak(usize), // minutes
	StartupAgain(StopwatchProgress),
	Stop,
	Pause,
	Resume,
	Toggle,
	CompleteTask,
	Update,
}

impl From<StopwatchPageMessage> for Message {
	fn from(value: StopwatchPageMessage) -> Self {
		Message::StopwatchPageMessage(value)
	}
}

impl StopwatchPage {
	pub fn clock(&self) -> Option<&StopwatchClock> {
		match self {
			StopwatchPage::StopTaskTime { clock, .. } => Some(clock),
			_ => None,
		}
	}

	pub fn subscription(&self, opened: bool) -> Subscription<StopwatchPageMessage> {
		let redraw_subscription = match self {
			StopwatchPage::Idle => Subscription::none(),
			StopwatchPage::StopTaskTime { .. } | StopwatchPage::TakingBreak { .. } => {
				if opened {
					window::frames().map(|_| StopwatchPageMessage::Update)
				} else {
					time::every(Duration::from_secs(1)).map(|_| StopwatchPageMessage::Update)
				}
			},
			StopwatchPage::TrackTime { .. } =>
				time::every(Duration::from_secs(1)).map(|_| StopwatchPageMessage::Update),
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
			StopwatchPage::StopTaskTime {
				elapsed_time,
				paused,
				project_id,
				task_id,
				finished_notification_sent,
				..
			} => Some(StopwatchProgress::Task {
				project_id: *project_id,
				task_id: *task_id,
				elapsed_time_seconds: elapsed_time.as_secs(),
				paused: *paused,
				finished_notification_sent: *finished_notification_sent,
			}),
			StopwatchPage::TrackTime { elapsed_time, paused, .. } => Some(StopwatchProgress::TrackTime {
				elapsed_time_seconds: elapsed_time.as_secs(),
				paused: *paused
			}),
			StopwatchPage::TakingBreak {
				elapsed_time,
				paused,
				break_duration_minutes,
				break_over_notification_sent,
				..
			} => Some(StopwatchProgress::Break {
				elapsed_time_seconds: elapsed_time.as_secs(),
				paused: *paused,
				break_duration_minutes: *break_duration_minutes,
				break_over_notification_sent: *break_over_notification_sent,
			})
		}
	}

	pub fn update(
		&mut self,
		message: StopwatchPageMessage,
		database: &Option<Database>,
		preferences: &Option<Preferences>,
		opened: bool,
	) -> Option<Vec<Message>> {
		let get_needed_seconds = |project_id: ProjectId, task_id: TaskId| -> Option<f32> {
			database.as_ref().and_then(|database|
				database.get_task(&project_id, &task_id)
					.and_then(|task|
						task.needed_time_minutes.as_ref()
							.map(|needed_minutes| *needed_minutes as f32 * 60.0)
					)
			)
		};

		match message {
			StopwatchPageMessage::StartTrackingTime => {
				*self = StopwatchPage::TrackTime {
					elapsed_time: Duration::from_secs(0),
					last_update: Instant::now(),
					paused: false
				};
				Some(vec![PreferenceMessage::SetStopwatchProgress(self.get_progress()).into()])
			},
			StopwatchPageMessage::StopTask { project_id, task_id } => {
				if let Some(needed_seconds) = get_needed_seconds(project_id, task_id) {
					*self = StopwatchPage::StopTaskTime {
						elapsed_time: Duration::ZERO,
						last_update: Instant::now(),
						paused: false,
						project_id,
						task_id,
						clock: StopwatchClock::new(0.0, needed_seconds, Some(needed_seconds)),
						finished_notification_sent: false,
					};
					Some(vec![
						Message::OpenStopwatch,
						PreferenceMessage::SetStopwatchProgress(self.get_progress()).into(),
					])
				}
				else {
					self.update(StopwatchPageMessage::StartTrackingTime, database, preferences, opened)
				}
			},
			StopwatchPageMessage::TakeBreak(minutes) => {
				let duration_seconds = minutes as f32 * 60.0;

				*self = StopwatchPage::TakingBreak {
					elapsed_time: Duration::from_secs(0),
					last_update: Instant::now(),
					paused: false,
					break_duration_minutes: minutes,
					break_over_notification_sent: false,
					clock: StopwatchClock::new(0.0, duration_seconds, None),
				};
				Some(vec![PreferenceMessage::SetStopwatchProgress(self.get_progress()).into()])
			},
			StopwatchPageMessage::StartupAgain(progress) => {
				*self = match progress {
					StopwatchProgress::TrackTime { elapsed_time_seconds, paused } => StopwatchPage::TrackTime{
						elapsed_time: Duration::from_secs(elapsed_time_seconds),
						paused,
						last_update: Instant::now()
					},
					StopwatchProgress::Task {
						project_id,
						task_id,
						elapsed_time_seconds,
						paused,
						finished_notification_sent
					} => {
						let elapsed_time = Duration::from_secs(elapsed_time_seconds);
						let last_update = Instant::now();

						if let Some(needed_seconds) = get_needed_seconds(project_id, task_id) {
							StopwatchPage::StopTaskTime{
								elapsed_time,
								project_id,
								task_id,
								paused,
								finished_notification_sent,
								last_update,
								clock: StopwatchClock::new(
									elapsed_time_seconds as f32 / needed_seconds,
									needed_seconds - elapsed_time_seconds as f32,
									Some(needed_seconds)
								)
							}
						}
						else {
							StopwatchPage::TrackTime {
								elapsed_time,
								last_update,
								paused
							}
						}
					},
					StopwatchProgress::Break {
						elapsed_time_seconds,
						paused,
						break_duration_minutes,
						break_over_notification_sent
					} => {
						let duration_seconds = break_duration_minutes as f32 * 60.0;

						StopwatchPage::TakingBreak {
							elapsed_time: Duration::from_secs(elapsed_time_seconds),
							last_update: Instant::now(),
							paused,
							break_duration_minutes,
							break_over_notification_sent,
							clock: StopwatchClock::new(
								elapsed_time_seconds as f32 / duration_seconds,
								duration_seconds - elapsed_time_seconds as f32,
								None
							)
						}
					}
				};
				None
			}
			StopwatchPageMessage::Stop => {
				*self = StopwatchPage::Idle;
				Some(vec![PreferenceMessage::SetStopwatchProgress(
					self.get_progress(),
				)
				.into()])
			}
			StopwatchPageMessage::Resume => match self {
				StopwatchPage::TrackTime { paused, .. } |
				StopwatchPage::StopTaskTime { paused, .. } |
				StopwatchPage::TakingBreak { paused, .. } => {
					*paused = false;
					Some(vec![PreferenceMessage::SetStopwatchProgress(
						self.get_progress(),
					)
					.into()])
				},
				StopwatchPage::Idle => None,
			}
			StopwatchPageMessage::Pause => match self {
				StopwatchPage::TrackTime { paused, .. } |
				StopwatchPage::StopTaskTime { paused, .. } |
				StopwatchPage::TakingBreak { paused, .. } => {
					*paused = true;
					Some(vec![PreferenceMessage::SetStopwatchProgress(
						self.get_progress(),
					)
					.into()])
				},
				StopwatchPage::Idle => None,
			}
			StopwatchPageMessage::Toggle => if opened {
				match self {
					StopwatchPage::TrackTime { paused, .. } |
					StopwatchPage::StopTaskTime { paused, .. } |
					StopwatchPage::TakingBreak { paused, .. } => {
						*paused = !*paused;
						Some(vec![PreferenceMessage::SetStopwatchProgress(
							self.get_progress(),
						)
						.into()])
					},
					StopwatchPage::Idle => self.update(StopwatchPageMessage::StartTrackingTime, database, preferences, opened),
				}
			}
			else {
				None
			},
			StopwatchPageMessage::CompleteTask => {
				let database_action: Option<Message> = if let StopwatchPage::StopTaskTime {
					project_id,
					task_id,
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
				let actions = self.update(StopwatchPageMessage::Stop, database, preferences, opened);

				if let Some(mut actions) = actions {
					if let Some(database_action) = database_action {
						actions.push(database_action);
					}
					Some(actions)
				} else {
					database_action.map(|db_action| vec![db_action])
				}
			},
			StopwatchPageMessage::Update => {
				// advance time
				match self {
					StopwatchPage::Idle => {},
					StopwatchPage::TrackTime { elapsed_time, last_update, paused } |
					StopwatchPage::StopTaskTime { elapsed_time, last_update, paused, .. } |
					StopwatchPage::TakingBreak { elapsed_time, last_update, paused, .. } => {
						if !*paused {
							*elapsed_time += Instant::now().duration_since(*last_update);
						}
						*last_update = Instant::now();
					},
				}

				// check if timer is finished
				match self {
					StopwatchPage::StopTaskTime { elapsed_time, project_id, task_id, clock, finished_notification_sent, .. } => {
						let task_and_type = database
							.as_ref()
							.and_then(|db| db.get_task_and_type(project_id, task_id));

						if let Some((task, task_type)) = task_and_type {
							if matches!(task_type, TaskType::Done) {
								*self = StopwatchPage::Idle;
							}
							else if let Some(needed_minutes) = task.needed_time_minutes {
								let timer_seconds = elapsed_time.as_secs_f32();
								let needed_seconds = needed_minutes as f32 * 60.0;
								let seconds_left = needed_seconds - timer_seconds;
								clock.set_percentage(timer_seconds / needed_seconds);
								clock.set_seconds_left(seconds_left);
								clock.set_needed_seconds(needed_seconds);

								if seconds_left <= 0.0 && !*finished_notification_sent {
									*finished_notification_sent = true;

									if preferences.play_timer_notification_sound() {
										timer_notification(format!("{} min. timer finished!", needed_minutes), task.name().clone());
									}
								}
							}
						}
						else {
							*self = StopwatchPage::Idle;
						}
					},
					StopwatchPage::TakingBreak { elapsed_time, break_duration_minutes, break_over_notification_sent, clock, .. } => {
						let timer_seconds = elapsed_time.as_secs_f32();
						let needed_seconds = *break_duration_minutes as f32 * 60.0;
						let seconds_left = needed_seconds - timer_seconds;
						clock.set_percentage(timer_seconds / needed_seconds);
						clock.set_seconds_left(seconds_left);
						// Empty, since we display a x min. break text below
						// clock.set_needed_seconds(needed_seconds);

						if seconds_left <= 0.0 && !*break_over_notification_sent {
							*break_over_notification_sent = true;

							if preferences.play_timer_notification_sound() {
								timer_notification(format!("{break_duration_minutes} min. break is over!"), "".to_string());
							}
						}
					},
					_ => {},
				}

				Some(vec![PreferenceMessage::SetStopwatchProgress(self.get_progress()).into()])
			}
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<'a, Message> {
		container(match self {
			StopwatchPage::Idle => Element::new(responsive(move |size| {
				let track_time = column![
					text("Track time:")
						.size(45),
					track_time_button()
				]
				.align_x(Alignment::Center)
				.spacing(LARGE_SPACING_AMOUNT);

				let take_break = column![
					text("or take a break:").size(45),
					row![
						take_break_button(5),
						take_break_button(15),
						take_break_button(30),
					]
					.spacing(LARGE_SPACING_AMOUNT)
				]
				.align_x(Alignment::Center)
				.spacing(LARGE_SPACING_AMOUNT);

				let page_view: Element<Message> = if size.width > size.height * 2.0 {
					row![
						track_time,
						take_break
					]
					.spacing(LARGE_SPACING_AMOUNT * 3)
					.into()
				}
				else {
					column![
						track_time,
						take_break,
					]
					.align_x(Alignment::Center)
					.spacing(LARGE_SPACING_AMOUNT)
					.into()
				};

				container(page_view)
					.center(Fill)
					.into()
			})),

			StopwatchPage::TrackTime { elapsed_time, paused, .. } => {
				column![
					text(format_stopwatch_duration(
						elapsed_time.as_secs_f64().round_ties_even() as i64,
					))
					.font(Font::DEFAULT)
					.size(90)
					.width(Fill)
					.align_x(Horizontal::Center),

					row![
						if *paused {
							resume_timer_button()
						} else {
							pause_timer_button()
						},
						stop_timer_button()
					]
					.spacing(LARGE_SPACING_AMOUNT)
				]
				.align_x(Alignment::Center)
				.spacing(LARGE_SPACING_AMOUNT)
				.width(Fill)
				.into()
			},

			StopwatchPage::TakingBreak { break_duration_minutes, paused, clock, .. } => {
				responsive(move |size| {
					let clock = canvas(clock)
						.width(Length::Fixed(225.0))
						.height(Length::Fixed(225.0));

					let controls = column![
						text(format!("{break_duration_minutes} min. break")).size(45),

						row![
							if *paused {
								resume_timer_button()
							} else {
								pause_timer_button()
							},
							stop_timer_button()
						]
						.spacing(LARGE_SPACING_AMOUNT)
					]
					.align_x(Alignment::Center)
					.spacing(LARGE_SPACING_AMOUNT);

					let page_view: Element<Message> = if size.width > size.height {
						row![clock, controls]
							.spacing(LARGE_SPACING_AMOUNT)
							.align_y(Vertical::Center)
							.into()
					}
					else {
						column![clock, controls]
							.spacing(LARGE_SPACING_AMOUNT)
							.align_x(Horizontal::Center)
							.into()
					};

					container(
						page_view
					)
					.center(Fill)
					.into()
				})
				.into()
			},

			StopwatchPage::StopTaskTime {
				project_id,
				task_id,
				clock,
				paused,
				..
			} => {
				let mut project_ref = None;
				let mut task_ref = None;
				if let Some(database) = &app.database {
					if let Some(project) = database.get_project(project_id) {
						project_ref = Some(project);
						task_ref = project.get_task(task_id);
					}
				}

				responsive(move |size| -> Element<Message> {
					let clock: Element<Message> = if task_ref.is_some() {
						canvas(clock)
							.width(Length::Fixed(300.0))
							.height(Length::Fixed(300.0))
							.into()
					} else {
						text("<invalid project or task id>").into()
					};

					let controls = row![
						if *paused {
							resume_timer_button()
						} else {
							pause_timer_button()
						},
						stop_timer_button(),
						complete_task_timer_button()
					]
					.spacing(LARGE_SPACING_AMOUNT);

					let page_view: Element<Message> = if let Some(task_info) = task_info(task_ref, project_ref, app) {
						if size.width > size.height {
							row![
								clock,
								column![
									task_info,
									controls,
								]
								.align_x(Alignment::Center)
								.spacing(LARGE_SPACING_AMOUNT)
								.width(Fill)
							]
							.spacing(LARGE_SPACING_AMOUNT)
							.align_y(Vertical::Center)
							.into()
						}
						else {
							column![
								clock,
								Space::new(0.0, LARGE_PADDING_AMOUNT),
								task_info,
								controls,
							]
							.spacing(LARGE_SPACING_AMOUNT)
							.align_x(Horizontal::Center)
							.into()
						}
					}
					else {
						column![
							clock,
							controls
						]
						.align_x(Alignment::Center)
						.spacing(LARGE_SPACING_AMOUNT)
						.width(Fill)
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

fn timer_notification(summary: String, body: String) {
	// play notification sound
	thread::spawn(|| {
		match rodio::OutputStream::try_default() {
			Ok((_stream, stream_handle)) => {
				let notification_sound_data = include_bytes!("../../assets/message-new-instant.oga");
				match stream_handle.play_once(Cursor::new(notification_sound_data)) {
					Ok(sink) => sink.sleep_until_end(),
					Err(e) => eprintln!("Failed to play notification sound: {e}"),
				}
			},
			Err(e) => eprintln!("Failed to play notification sound: {e}"),
		}
	});

	// show notification
	let notification_result = if cfg!(target_os = "linux") {
		use notify_rust::Hint;

		Notification::new()
			.summary(&summary)
			.body(&body)
			.appname("Project Tracker")
			.icon("Project Tracker")
			.hint(Hint::DesktopEntry("Project Tracker".to_string()))
			.show()
	}
	else {
		Notification::new()
			.summary(&summary)
			.body(&body)
			.show()
	};

	match notification_result {
		Ok(notification_handle) => notification_handle.on_close(|| {}),
		Err(e) => eprintln!("failed to show timer notification: {e}"),
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
