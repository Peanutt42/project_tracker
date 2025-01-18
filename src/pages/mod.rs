use crate::{project_tracker, Preferences, ProjectTrackerApp, SerializedContentPage};
use iced::{Element, Subscription};
use project_tracker_core::{Database, DatabaseMessage, ProjectId, TaskId};

pub mod sidebar_page;
pub use sidebar_page::{TaskDropzone, BOTTOM_TODO_TASK_DROPZONE_ID, STOPWATCH_TASK_DROPZONE_ID};

pub mod project_page;
pub use project_page::CachedTaskList;

pub mod stopwatch_page;
pub use stopwatch_page::format_stopwatch_duration;

pub mod overview_page;

#[derive(Debug)]
pub struct Page {
	pub overview_page: Option<overview_page::Page>,
	pub stopwatch_page: stopwatch_page::Page,
	pub project_page: Option<project_page::Page>,
}

#[derive(Debug, Clone)]
pub enum Message {
	StopwatchPage(stopwatch_page::Message),
	ProjectPage(project_page::Message),
	OverviewPage(overview_page::Message),
	OpenOverview,
	OpenProjectPage(ProjectId),
	OpenStopwatch,
}

impl From<Message> for project_tracker::Message {
	fn from(value: Message) -> Self {
		project_tracker::Message::ContentPageMessage(value)
	}
}

#[must_use]
#[derive(Default)]
pub enum Action {
	#[default]
	None,
	Actions(Vec<Action>),
	Task(iced::Task<Message>),
	DatabaseMessage(DatabaseMessage),
	OpenManageTaskTagsModal(ProjectId),
	ConfirmDeleteProject {
		project_id: ProjectId,
		project_name: String,
	},
	OpenTaskModal {
		project_id: ProjectId,
		task_id: TaskId,
	},
	CloseTaskModal,
	OpenStopwatch,
}

impl From<iced::Task<Message>> for Action {
	fn from(value: iced::Task<Message>) -> Self {
		Action::Task(value)
	}
}

impl From<DatabaseMessage> for Action {
	fn from(value: DatabaseMessage) -> Self {
		Action::DatabaseMessage(value)
	}
}

impl Page {
	pub fn new(database: Option<&Database>, preferences: &Option<Preferences>) -> Self {
		Self {
			overview_page: Some(overview_page::Page::new(database, preferences)),
			stopwatch_page: stopwatch_page::Page::default(),
			project_page: None,
		}
	}

	pub fn restore_from_serialized(
		&mut self,
		database: Option<&Database>,
		preferences: &mut Option<Preferences>,
	) -> Action {
		match preferences {
			Some(ref_preferences) => {
				let action = match ref_preferences.stopwatch_progress() {
					Some(stopwatch_progress) => {
						let (stopwatch_page, action) =
							stopwatch_page::Page::startup_again(*stopwatch_progress, database);
						self.stopwatch_page = stopwatch_page;
						action
					}
					None => Action::None,
				};

				match ref_preferences.selected_content_page() {
					SerializedContentPage::Overview => self.open_overview(database, preferences),
					SerializedContentPage::Stopwatch => self.open_stopwatch(preferences),
					SerializedContentPage::Project(project_id) => {
						let project_id_to_open = match &self.project_page {
							Some(project_page) => project_page.project_id,
							None => *project_id,
						};
						self.open_project_page(project_id_to_open, database, preferences);
					}
				}

				action
			}
			None => Action::None,
		}
	}

	pub fn is_overview_page_opened(&self) -> bool {
		self.overview_page.is_some()
	}

	pub fn is_project_page_opened(&self) -> bool {
		self.project_page.is_some()
	}

	pub fn is_stopwatch_page_opened(&self) -> bool {
		self.overview_page.is_none() && self.project_page.is_none()
	}

	pub fn subscription(&self) -> Subscription<Message> {
		Subscription::batch([
			self.stopwatch_page
				.subscription(self.project_page.is_none())
				.map(Message::StopwatchPage),
			match &self.project_page {
				Some(project_page) => project_page.subscription().map(Message::ProjectPage),
				None => Subscription::none(),
			},
		])
	}

	pub fn update(
		&mut self,
		message: Message,
		database: Option<&Database>,
		preferences: &mut Option<Preferences>,
	) -> Action {
		match message {
			Message::ProjectPage(message) => match &mut self.project_page {
				Some(project_page) => project_page.update(message, database, preferences),
				None => Action::None,
			},
			Message::StopwatchPage(message) => {
				let mut actions = Vec::new();
				if matches!(message, stopwatch_page::Message::StopTask { .. }) {
					actions.push(Action::CloseTaskModal);
				};
				actions.push(self.stopwatch_page.update(
					message,
					database,
					preferences,
					self.is_stopwatch_page_opened(),
				));
				Action::Actions(actions)
			}
			Message::OverviewPage(message) => {
				if let Some(overview_page) = &mut self.overview_page {
					overview_page.update(message, database, preferences);
				}
				Action::None
			}
			Message::OpenOverview => {
				self.open_overview(database, preferences);
				Action::None
			}
			Message::OpenProjectPage(project_id) => {
				self.open_project_page(project_id, database, preferences);
				Action::None
			}
			Message::OpenStopwatch => {
				self.open_stopwatch(preferences);
				Action::None
			}
		}
	}

	fn open_overview(
		&mut self,
		database: Option<&Database>,
		preferences: &mut Option<Preferences>,
	) {
		self.project_page = None;
		self.overview_page = Some(overview_page::Page::new(database, preferences));
		if let Some(preferences) = preferences {
			preferences.set_selected_content_page(SerializedContentPage::Overview);
		}
	}

	fn open_project_page(
		&mut self,
		project_id: ProjectId,
		database: Option<&Database>,
		preferences: &mut Option<Preferences>,
	) {
		self.overview_page = None;
		let open_project_info = database.as_ref().and_then(|database| {
			database
				.get_project(&project_id)
				.map(|project| (project_id, project))
		});
		match open_project_info {
			Some((project_id, project)) => {
				self.project_page = Some(project_page::Page::new(project_id, project, preferences));
				if let Some(preferences) = preferences {
					preferences
						.set_selected_content_page(SerializedContentPage::Project(project_id));
				}
			}
			None if database.is_some() => self.open_stopwatch(preferences),
			None => {
				// database is not loaded yet -> dont override saved selected content page yet
				self.overview_page = None;
				self.project_page = None;
			}
		}
	}

	fn open_stopwatch(&mut self, preferences: &mut Option<Preferences>) {
		self.overview_page = None;
		self.project_page = None;
		if let Some(preferences) = preferences {
			preferences.set_selected_content_page(SerializedContentPage::Stopwatch);
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<'a, project_tracker::Message> {
		match &self.project_page {
			Some(project_page) => project_page.view(app),
			None => match &self.overview_page {
				Some(overview_page) => overview_page.view(app),
				None => self.stopwatch_page.view(app),
			},
		}
	}
}
