use crate::{
	components::{
		complete_task_timer_button, days_left_widget, horizontal_scrollable, loading_screen,
		open_project_button, open_task_by_name_link_button, pause_timer_button,
		resume_timer_button, stop_timer_button, take_break_button, task_description,
		track_time_button, StopwatchClock, HORIZONTAL_SCROLLABLE_PADDING,
		LARGE_LOADING_SPINNER_SIZE,
	},
	core::IcedColorConversion,
	pages, project_tracker,
	styles::{
		task_tag_container_style, JET_BRAINS_MONO_FONT, LARGE_PADDING_AMOUNT, LARGE_SPACING_AMOUNT,
		PADDING_AMOUNT, SMALL_PADDING_AMOUNT, SPACING_AMOUNT,
	},
	DatabaseState, OptionalPreference, Preferences, ProjectTrackerApp, StopwatchProgress,
};
use iced::{
	alignment::{Horizontal, Vertical},
	keyboard, time,
	widget::{canvas, column, container, responsive, row, text, Column, Row, Space},
	window, Alignment, Element,
	Length::{self, Fill, Fixed},
	Padding, Subscription,
};
use project_tracker_core::{Database, DatabaseMessage, Project, ProjectId, Task, TaskId, TaskType};
use std::time::{Duration, Instant};
use tracing::error;

#[derive(Debug, Default)]
pub enum Page {
	#[default]
	Idle,
	TrackTime {
		elapsed_time: Duration,
		last_update: Instant,
		paused: bool,
	},
	StopTaskTime {
		paused: bool,
		project_id: ProjectId,
		task_id: TaskId,
		clock: Option<StopwatchClock>,
		finished_notification_sent: bool,
	},
	TakingBreak {
		elapsed_time: Duration,
		last_update: Instant,
		paused: bool,
		break_duration_minutes: usize,
		clock: StopwatchClock,
		break_over_notification_sent: bool,
	},
}

#[derive(Clone, Debug)]
pub enum Message {
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
	SaveTaskTimeSpendBeforeClosing,
}

impl From<Message> for project_tracker::Message {
	fn from(value: Message) -> Self {
		pages::Message::StopwatchPage(value).into()
	}
}

impl Page {
	pub fn startup_again(
		stopwatch_progress: StopwatchProgress,
		database: Option<&Database>,
	) -> (Self, pages::Action) {
		match stopwatch_progress {
			StopwatchProgress::TrackTime {
				elapsed_time_seconds,
				paused,
			} => (
				Page::TrackTime {
					elapsed_time: Duration::from_secs(elapsed_time_seconds),
					paused,
					last_update: Instant::now(),
				},
				pages::Action::None,
			),
			StopwatchProgress::Task {
				project_id,
				task_id,
				paused,
				finished_notification_sent,
			} => {
				let time_spend_seconds =
					Self::get_spend_seconds(project_id, task_id, database).unwrap_or(0.0);
				let needed_seconds = Self::get_needed_seconds(project_id, task_id, database);

				(
					Page::StopTaskTime {
						project_id,
						task_id,
						paused,
						finished_notification_sent,
						clock: needed_seconds.map(|needed_seconds| {
							StopwatchClock::new(
								time_spend_seconds / needed_seconds,
								needed_seconds - time_spend_seconds,
								Some(needed_seconds),
							)
						}),
					},
					if paused {
						pages::Action::None
					} else {
						DatabaseMessage::StartTaskTimeSpend {
							project_id,
							task_id,
						}
						.into()
					},
				)
			}
			StopwatchProgress::Break {
				elapsed_time_seconds,
				paused,
				break_duration_minutes,
				break_over_notification_sent,
			} => {
				let duration_seconds = break_duration_minutes as f32 * 60.0;

				(
					Page::TakingBreak {
						elapsed_time: Duration::from_secs(elapsed_time_seconds),
						last_update: Instant::now(),
						paused,
						break_duration_minutes,
						break_over_notification_sent,
						clock: StopwatchClock::new(
							elapsed_time_seconds as f32 / duration_seconds,
							duration_seconds - elapsed_time_seconds as f32,
							None,
						),
					},
					pages::Action::None,
				)
			}
		}
	}

	pub fn subscription(&self, opened: bool) -> Subscription<Message> {
		let redraw_subscription = match self {
			Page::Idle => Subscription::none(),
			Page::StopTaskTime { .. } | Page::TakingBreak { .. } => {
				if opened {
					window::frames().map(|_| Message::Update)
				} else {
					time::every(Duration::from_secs(1)).map(|_| Message::Update)
				}
			}
			Page::TrackTime { .. } => time::every(Duration::from_secs(1)).map(|_| Message::Update),
		};

		let toggle_subscription = keyboard::on_key_press(|key, modifiers| match key.as_ref() {
			keyboard::Key::Named(keyboard::key::Named::Space)
				if !modifiers.command() && !modifiers.shift() =>
			{
				Some(Message::Toggle)
			}
			_ => None,
		});

		Subscription::batch([redraw_subscription, toggle_subscription])
	}

	pub fn is_task_being_stopped(&self, project_id: ProjectId, task_id: TaskId) -> bool {
		match &self {
			Page::StopTaskTime {
				paused,
				project_id: stopped_project_id,
				task_id: stopped_task_id,
				..
			} if *stopped_project_id == project_id && *stopped_task_id == task_id => !*paused,
			_ => false,
		}
	}

	pub fn get_needed_seconds(
		project_id: ProjectId,
		task_id: TaskId,
		database: Option<&Database>,
	) -> Option<f32> {
		database.and_then(|database| {
			database.get_task(&project_id, &task_id).and_then(|task| {
				task.needed_time_minutes
					.as_ref()
					.map(|needed_minutes| *needed_minutes as f32 * 60.0)
			})
		})
	}

	pub fn get_spend_seconds(
		project_id: ProjectId,
		task_id: TaskId,
		database: Option<&Database>,
	) -> Option<f32> {
		database.and_then(|database| {
			database.get_task(&project_id, &task_id).and_then(|task| {
				task.time_spend
					.as_ref()
					.map(|time_spend| time_spend.get_seconds())
			})
		})
	}

	pub fn update(
		&mut self,
		message: Message,
		database: Option<&Database>,
		preferences: &mut Option<Preferences>,
		opened: bool,
	) -> pages::Action {
		match message {
			Message::StartTrackingTime => {
				*self = Page::TrackTime {
					elapsed_time: Duration::from_secs(0),
					last_update: Instant::now(),
					paused: false,
				};
				self.set_stopwatch_progress(preferences);
				pages::Action::None
			}
			Message::StopTask {
				project_id,
				task_id,
			} => {
				*self = Page::StopTaskTime {
					paused: false,
					project_id,
					task_id,
					clock: Self::get_needed_seconds(project_id, task_id, database).map(
						|needed_seconds| {
							StopwatchClock::new(0.0, needed_seconds, Some(needed_seconds))
						},
					),
					finished_notification_sent: false,
				};
				self.set_stopwatch_progress(preferences);
				pages::Action::Actions(vec![
					pages::Action::OpenStopwatch,
					DatabaseMessage::StartTaskTimeSpend {
						project_id,
						task_id,
					}
					.into(),
				])
			}
			Message::TakeBreak(minutes) => {
				let duration_seconds = minutes as f32 * 60.0;

				*self = Page::TakingBreak {
					elapsed_time: Duration::from_secs(0),
					last_update: Instant::now(),
					paused: false,
					break_duration_minutes: minutes,
					break_over_notification_sent: false,
					clock: StopwatchClock::new(0.0, duration_seconds, None),
				};
				self.set_stopwatch_progress(preferences);
				pages::Action::None
			}
			Message::StartupAgain(progress) => {
				let (new_self, action) = Self::startup_again(progress, database);
				*self = new_self;
				action
			}
			Message::Stop => self.stop(preferences),
			Message::Resume => match self {
				Page::TrackTime { paused, .. } | Page::TakingBreak { paused, .. } => {
					*paused = false;
					self.set_stopwatch_progress(preferences);
					pages::Action::None
				}
				Page::StopTaskTime {
					project_id,
					task_id,
					paused,
					..
				} => {
					*paused = false;
					let project_id = *project_id;
					let task_id = *task_id;
					self.set_stopwatch_progress(preferences);

					DatabaseMessage::StartTaskTimeSpend {
						project_id,
						task_id,
					}
					.into()
				}
				Page::Idle => pages::Action::None,
			},
			Message::Pause => match self {
				Page::TrackTime { paused, .. } | Page::TakingBreak { paused, .. } => {
					*paused = true;
					self.set_stopwatch_progress(preferences);
					pages::Action::None
				}
				Page::StopTaskTime {
					project_id,
					task_id,
					paused,
					..
				} => {
					*paused = true;
					let project_id = *project_id;
					let task_id = *task_id;
					self.set_stopwatch_progress(preferences);

					DatabaseMessage::StopTaskTimeSpend {
						project_id,
						task_id,
					}
					.into()
				}
				Page::Idle => pages::Action::None,
			},
			Message::Toggle => {
				if opened {
					match self {
						Page::TrackTime { paused, .. } | Page::TakingBreak { paused, .. } => {
							*paused = !*paused;
							self.set_stopwatch_progress(preferences);
							pages::Action::None
						}
						Page::StopTaskTime {
							paused,
							project_id,
							task_id,
							..
						} => {
							// resume
							let action = if *paused {
								DatabaseMessage::StartTaskTimeSpend {
									project_id: *project_id,
									task_id: *task_id,
								}
								.into()
							} else {
								// pause
								DatabaseMessage::StopTaskTimeSpend {
									project_id: *project_id,
									task_id: *task_id,
								}
								.into()
							};
							*paused = !*paused;
							self.set_stopwatch_progress(preferences);
							action
						}
						Page::Idle => {
							self.update(Message::StartTrackingTime, database, preferences, opened)
						}
					}
				} else {
					pages::Action::None
				}
			}
			Message::CompleteTask => {
				let set_task_done_action = match self {
					Page::StopTaskTime {
						project_id,
						task_id,
						..
					} => DatabaseMessage::SetTaskDone {
						project_id: *project_id,
						task_id: *task_id,
					}
					.into(),
					_ => pages::Action::None,
				};
				pages::Action::Actions(vec![set_task_done_action, self.stop(preferences)])
			}
			Message::Update => {
				// advance time
				match self {
					// stop_task_time stores the start time to get the elapsed time
					Page::Idle | Page::StopTaskTime { .. } => {}
					Page::TrackTime {
						elapsed_time,
						last_update,
						paused,
					}
					| Page::TakingBreak {
						elapsed_time,
						last_update,
						paused,
						..
					} => {
						if !*paused {
							*elapsed_time += Instant::now().duration_since(*last_update);
						}
						*last_update = Instant::now();
					}
				}

				// check if timer is finished
				match self {
					Page::StopTaskTime {
						project_id,
						task_id,
						clock,
						finished_notification_sent,
						..
					} => {
						let task_and_type = database
							.as_ref()
							.and_then(|db| db.get_task_and_type(project_id, task_id));

						match task_and_type {
							Some((task, task_type)) => {
								if matches!(task_type, TaskType::Done) {
									*self = Page::Idle;
								} else {
									match task.needed_time_minutes {
										Some(needed_minutes) => {
											let timer_seconds = Self::get_spend_seconds(
												*project_id,
												*task_id,
												database,
											)
											.unwrap_or(0.0);
											let needed_seconds = needed_minutes as f32 * 60.0;
											let seconds_left = needed_seconds - timer_seconds;
											let percentage = timer_seconds / needed_seconds;

											match clock {
												Some(clock) => {
													clock.set_percentage(percentage);
													clock.set_seconds_left(seconds_left);
													clock.set_needed_seconds(needed_seconds);
												}
												None => {
													*clock = Some(StopwatchClock::new(
														percentage,
														seconds_left,
														Some(needed_seconds),
													));
												}
											}

											if seconds_left <= 0.0 && !*finished_notification_sent {
												*finished_notification_sent = true;

												if preferences.play_timer_notification_sound() {
													timer_notification(
														format!(
															"{} min. timer finished!",
															needed_minutes
														),
														task.name.clone(),
													);
												}
											}
										}
										None => *clock = None,
									}
								}
							}
							None => *self = Page::Idle,
						}
					}
					Page::TakingBreak {
						elapsed_time,
						break_duration_minutes,
						break_over_notification_sent,
						clock,
						..
					} => {
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
								timer_notification(
									format!("{break_duration_minutes} min. break is over!"),
									"".to_string(),
								);
							}
						}
					}
					_ => {}
				}

				self.set_stopwatch_progress(preferences);

				pages::Action::None
			}
			Message::SaveTaskTimeSpendBeforeClosing => match self {
				Page::StopTaskTime {
					project_id,
					task_id,
					..
				} => DatabaseMessage::StopTaskTimeSpend {
					project_id: *project_id,
					task_id: *task_id,
				}
				.into(),
				_ => pages::Action::None,
			},
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<'a, project_tracker::Message> {
		container(match self {
			Page::Idle => Element::new(responsive(move |size| {
				let track_time = column![text("Track time:").size(45), track_time_button()]
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

				let page_view: Element<project_tracker::Message> = if size.width > size.height * 2.0
				{
					row![track_time, take_break]
						.spacing(LARGE_SPACING_AMOUNT * 3)
						.into()
				} else {
					column![track_time, take_break,]
						.align_x(Alignment::Center)
						.spacing(LARGE_SPACING_AMOUNT)
						.into()
				};

				container(page_view).center(Fill).into()
			})),

			Page::TrackTime {
				elapsed_time,
				paused,
				..
			} => column![
				text(format_stopwatch_duration(
					elapsed_time.as_secs_f64().round_ties_even() as i64,
				))
				.font(JET_BRAINS_MONO_FONT)
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
			.into(),

			Page::TakingBreak {
				break_duration_minutes,
				paused,
				clock,
				..
			} => responsive(move |size| {
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

				let page_view: Element<project_tracker::Message> = if size.width > size.height {
					row![clock, controls]
						.spacing(LARGE_SPACING_AMOUNT)
						.align_y(Vertical::Center)
						.into()
				} else {
					column![clock, controls]
						.spacing(LARGE_SPACING_AMOUNT)
						.align_x(Horizontal::Center)
						.into()
				};

				container(page_view).center(Fill).into()
			})
			.into(),

			Page::StopTaskTime {
				project_id,
				task_id,
				clock,
				paused,
				..
			} => {
				let mut project_ref = None;
				let mut task_ref = None;
				if let DatabaseState::Loaded(database) = &app.database {
					if let Some(project) = database.get_project(project_id) {
						project_ref = Some(project);
						task_ref = project.get_task(task_id);
					}
				}

				responsive(move |size| -> Element<project_tracker::Message> {
					let clock: Element<project_tracker::Message> = if task_ref.is_some() {
						match clock {
							Some(clock) => canvas(clock)
								.width(Fixed(300.0))
								.height(Fixed(300.0))
								.into(),
							None => text(format_stopwatch_duration(
								Self::get_spend_seconds(*project_id, *task_id, app.database.ok())
									.unwrap_or(0.0)
									.round_ties_even() as i64,
							))
							.font(JET_BRAINS_MONO_FONT)
							.size(90)
							.width(Fill)
							.align_x(Horizontal::Center)
							.into(),
						}
					} else if app.database.is_loaded() {
						error!("invalid project_id or task_id: doesnt exist in database!");
						text("<invalid project or task id>").into()
					} else {
						// db is still loading
						container(loading_screen(LARGE_LOADING_SPINNER_SIZE))
							.center(Fixed(300.0))
							.into()
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

					let page_view: Element<project_tracker::Message> = match task_info(
						task_ref,
						*task_id,
						project_ref.map(|project_ref| (*project_id, project_ref)),
						app,
					) {
						Some(task_info) => {
							if size.width > size.height {
								row![
									clock,
									column![task_info, controls,]
										.align_x(Alignment::Center)
										.spacing(LARGE_SPACING_AMOUNT)
										.width(Fill)
								]
								.spacing(LARGE_SPACING_AMOUNT)
								.align_y(Vertical::Center)
								.into()
							} else {
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
						None => column![clock, controls]
							.align_x(Alignment::Center)
							.spacing(LARGE_SPACING_AMOUNT)
							.width(Fill)
							.into(),
					};

					container(page_view).center(Fill).into()
				})
				.into()
			}
		})
		.center(Fill)
		.padding(LARGE_PADDING_AMOUNT)
		.into()
	}

	fn stop(&mut self, preferences: &mut Option<Preferences>) -> pages::Action {
		let action = match self {
			Page::StopTaskTime {
				project_id,
				task_id,
				..
			} => DatabaseMessage::StopTaskTimeSpend {
				project_id: *project_id,
				task_id: *task_id,
			}
			.into(),
			_ => pages::Action::None,
		};
		*self = Page::Idle;
		self.set_stopwatch_progress(preferences);
		action
	}

	fn set_stopwatch_progress(&self, preferences: &mut Option<Preferences>) {
		if let Some(preferences) = preferences {
			let progress = match self {
				Page::Idle => None,
				Page::StopTaskTime {
					paused,
					project_id,
					task_id,
					finished_notification_sent,
					..
				} => Some(StopwatchProgress::Task {
					project_id: *project_id,
					task_id: *task_id,
					paused: *paused,
					finished_notification_sent: *finished_notification_sent,
				}),
				Page::TrackTime {
					elapsed_time,
					paused,
					..
				} => Some(StopwatchProgress::TrackTime {
					elapsed_time_seconds: elapsed_time.as_secs(),
					paused: *paused,
				}),
				Page::TakingBreak {
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
				}),
			};
			preferences.set_stopwatch_progress(progress);
		}
	}
}

#[cfg(target_os = "windows")]
fn timer_notification(summary: String, body: String) {
	use std::path::Path;

	use winrt_notification::{Duration, IconCrop, Sound, Toast};

	let notification_result = Toast::new(Toast::POWERSHELL_APP_ID)
		.title(&summary)
		.text1(&body)
		.icon(
			Path::new(
				"C:\\Users\\madca\\AppData\\Local\\Programs\\Project Tracker\\ProjectTracker.ico",
			),
			IconCrop::Square,
			"Project Tracker Icon",
		)
		.duration(Duration::Long)
		.sound(Some(Sound::Reminder))
		.show();

	if let Err(e) = notification_result {
		error!("failed to show timer notification: {e}");
	}
}

#[cfg(not(target_os = "windows"))]
fn timer_notification(summary: String, body: String) {
	use notify_rust::{Notification, Timeout};
	use std::{io::Cursor, thread};

	// play notification sound
	thread::spawn(|| match rodio::OutputStream::try_default() {
		Ok((_stream, stream_handle)) => {
			let notification_sound_data = include_bytes!("../../assets/message-new-instant.oga");
			match stream_handle.play_once(Cursor::new(notification_sound_data)) {
				Ok(sink) => sink.sleep_until_end(),
				Err(e) => error!("Failed to play notification sound: {e}"),
			}
		}
		Err(e) => error!("Failed to play notification sound: {e}"),
	});

	// show notification
	let mut notification = Notification::new();
	notification
		.summary(&summary)
		.body(&body)
		.appname("project_tracker") // only used to display app name text inside the notification, nothing else
		.icon("project_tracker") // will resolve into 'project_tracker.png'
		.timeout(Timeout::Never);

	#[cfg(target_os = "linux")]
	notification.hint(notify_rust::Hint::DesktopEntry(
		"project_tracker".to_string(),
	));

	#[cfg(target_os = "linux")]
	notification.hint(notify_rust::Hint::Resident(true));

	#[allow(unused)]
	let notification_result = notification.show();

	#[cfg(target_os = "linux")]
	thread::spawn(|| match notification_result {
		Ok(notification_handle) => notification_handle.on_close(|| {}),
		Err(e) => error!("failed to show timer notification: {e}"),
	});
}

fn task_info<'a>(
	task: Option<&'a Task>,
	task_id: TaskId,
	project: Option<(ProjectId, &'a Project)>,
	app: &'a ProjectTrackerApp,
) -> Option<Element<'a, project_tracker::Message>> {
	task.map(|task| {
		Column::new()
			.push_maybe(project.as_ref().map(|(project_id, _project)| {
				open_task_by_name_link_button(*project_id, task_id, &task.name)
			}))
			.push_maybe(task.due_date.map(|date| days_left_widget(date, false)))
			.push_maybe(project.as_ref().map(|(project_id, _project)| {
				task_description(
					*project_id,
					task_id,
					app.task_description_markdown_storage.get(
						*project_id,
						task_id,
						app.database.ok(),
					),
					app,
				)
			}))
			.push_maybe(project.map(|(project_id, project)| {
				row![
					container(open_project_button(
						project_id,
						&project.name,
						project.color.to_iced_color()
					))
					.padding(HORIZONTAL_SCROLLABLE_PADDING),
					horizontal_scrollable(
						Row::with_children(task.tags.iter().map(|tag_id| {
							match project.task_tags.get(tag_id) {
								Some(tag) => container(text(&tag.name))
									.style(|t| {
										task_tag_container_style(t, tag.color.to_iced_color())
									})
									.padding(
										Padding::new(SMALL_PADDING_AMOUNT)
											.left(PADDING_AMOUNT)
											.right(PADDING_AMOUNT),
									)
									.into(),
								None => {
									if app.database.is_loaded() {
										error!("invalid tag_id: doesnt exist in database!");
										"<invalid tag id>".into()
									} else {
										// db still loading
										Space::new(0.0, 0.0).into()
									}
								}
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
		format!("{factor_str}{hours}:{minutes:0>2}:{seconds:0>2}")
	} else {
		format!("{factor_str}{minutes:0>2}:{seconds:0>2}")
	}
}
