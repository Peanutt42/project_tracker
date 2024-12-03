use iced::{Element, Subscription};
use crate::{project_tracker::Message, ProjectTrackerApp, Preferences, SerializedContentPage};
use project_tracker_core::{Database, DatabaseMessage, ProjectId, TaskId};

mod sidebar_page;
pub use sidebar_page::{
	SidebarPage, SidebarPageAction, SidebarPageMessage, TaskDropzone, BOTTOM_TODO_TASK_DROPZONE_ID,
	STOPWATCH_TASK_DROPZONE_ID,
};

mod project_page;
pub use project_page::{CachedTaskList, ProjectPage, ProjectPageMessage};

mod stopwatch_page;
pub use stopwatch_page::{format_stopwatch_duration, StopwatchPage, StopwatchPageMessage};

mod overview_page;
pub use overview_page::{OverviewPage, OverviewPageMessage};

#[derive(Debug)]
pub struct ContentPage {
	pub overview_page: Option<OverviewPage>,
	pub stopwatch_page: StopwatchPage,
	pub project_page: Option<ProjectPage>,
}

#[derive(Debug, Clone)]
pub enum ContentPageMessage {
	StopwatchPageMessage(StopwatchPageMessage),
	ProjectPageMessage(ProjectPageMessage),
	OverviewPageMessage(OverviewPageMessage),
	OpenOverview,
	OpenProjectPage(ProjectId),
	OpenStopwatch,
}

impl From<ContentPageMessage> for Message {
	fn from(value: ContentPageMessage) -> Self {
		Message::ContentPageMessage(value)
	}
}

#[must_use]
#[derive(Default)]
pub enum ContentPageAction {
	#[default]
	None,
	Actions(Vec<ContentPageAction>),
	Task(iced::Task<Message>),
	DatabaseMessage(DatabaseMessage),
	OpenManageTaskTagsModal(ProjectId),
	ConfirmDeleteProject{
		project_id: ProjectId,
		project_name: String,
	},
	OpenTaskModal{
		project_id: ProjectId,
		task_id: TaskId,
	},
	CloseTaskModal,
	OpenStopwatch,
}

impl From<iced::Task<Message>> for ContentPageAction {
	fn from(value: iced::Task<Message>) -> Self {
		ContentPageAction::Task(value)
	}
}

impl From<DatabaseMessage> for ContentPageAction {
	fn from(value: DatabaseMessage) -> Self {
		ContentPageAction::DatabaseMessage(value)
	}
}

impl ContentPage {
	pub fn new(database: &Option<Database>, preferences: &Option<Preferences>) -> Self {
		Self {
			overview_page: Some(OverviewPage::new(database, preferences)),
			stopwatch_page: StopwatchPage::default(),
			project_page: None,
		}
	}

	pub fn restore_from_serialized(&mut self, database: &Option<Database>, preferences: &mut Option<Preferences>) -> ContentPageAction {
		if let Some(ref_preferences) = preferences {
			let action = if let Some(stopwatch_progress) = ref_preferences.stopwatch_progress() {
				let (stopwatch_page, action) = StopwatchPage::startup_again(*stopwatch_progress, database);
				self.stopwatch_page = stopwatch_page;
				action
			}
			else {
				ContentPageAction::None
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
		else {
			ContentPageAction::None
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

	pub fn subscription(&self) -> Subscription<ContentPageMessage> {
		Subscription::batch([
			self.stopwatch_page
				.subscription(self.project_page.is_none())
				.map(ContentPageMessage::StopwatchPageMessage),

			if let Some(project_page) = &self.project_page {
				project_page
					.subscription()
					.map(ContentPageMessage::ProjectPageMessage)
			} else {
				Subscription::none()
			},
		])
	}

	pub fn update(&mut self, message: ContentPageMessage, database: &Option<Database>, preferences: &mut Option<Preferences>) -> ContentPageAction {
		match message {
			ContentPageMessage::ProjectPageMessage(message) => if let Some(project_page) = &mut self.project_page {
				project_page.update(message, database, preferences)
			}
			else {
				ContentPageAction::None
			},
			ContentPageMessage::StopwatchPageMessage(message) => {
				let mut actions = Vec::new();
				if matches!(message, StopwatchPageMessage::StopTask { .. }) {
					actions.push(ContentPageAction::CloseTaskModal);
				};
				actions.push(self.stopwatch_page.update(message, database, preferences, self.is_stopwatch_page_opened()));
				ContentPageAction::Actions(actions)
			},
			ContentPageMessage::OverviewPageMessage(message) => {
				if let Some(overview_page) = &mut self.overview_page {
					overview_page.update(message, database, preferences);
				}
				ContentPageAction::None
			}
			ContentPageMessage::OpenOverview => {
				self.open_overview(database, preferences);
				ContentPageAction::None
			},
			ContentPageMessage::OpenProjectPage(project_id) => {
				self.open_project_page(project_id, database, preferences);
				ContentPageAction::None
			},
			ContentPageMessage::OpenStopwatch => {
				self.open_stopwatch(preferences);
				ContentPageAction::None
			}
		}
	}

	fn open_overview(&mut self, database: &Option<Database>, preferences: &mut Option<Preferences>) {
		self.project_page = None;
		self.overview_page = Some(OverviewPage::new(database, preferences));
		if let Some(preferences) = preferences {
			preferences.set_selected_content_page(SerializedContentPage::Overview);
		}
	}

	fn open_project_page(&mut self, project_id: ProjectId, database: &Option<Database>, preferences: &mut Option<Preferences>) {
		self.overview_page = None;
		let open_project_info = database.as_ref().and_then(|database|
			database
				.get_project(&project_id)
				.map(|project| (project_id, project))
		);
		if let Some((project_id, project)) = open_project_info {
			self.project_page = Some(ProjectPage::new(project_id, project, preferences));
			if let Some(preferences) = preferences {
				preferences.set_selected_content_page(SerializedContentPage::Project(project_id));
			}
		} else {
			self.open_stopwatch(preferences);
		}
	}

	fn open_stopwatch(&mut self, preferences: &mut Option<Preferences>) {
		self.overview_page = None;
		self.project_page = None;
		if let Some(preferences) = preferences {
			preferences.set_selected_content_page(SerializedContentPage::Stopwatch);
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<'a, Message> {
		if let Some(project_page) = &self.project_page {
			project_page.view(app)
		}
		else if let Some(overview_page) = &self.overview_page {
			overview_page.view(app)
		}
		else {
			self.stopwatch_page.view(app)
		}
	}
}