mod sidebar_page;
pub use sidebar_page::{SidebarPage, SidebarPageMessage, TaskDropzone, BOTTOM_TODO_TASK_DROPZONE_ID};

mod project_page;
pub use project_page::{ProjectPage, ProjectPageMessage, EditTaskState, CachedTaskList};

mod overview_page;
pub use overview_page::OverviewPage;

use iced::Element;
use crate::project_tracker::{ProjectTrackerApp, UiMessage};

pub enum ContentPage {
	Project(Box<ProjectPage>),
	Overview(OverviewPage),
}

impl ContentPage {
	pub fn is_overview_page(&self) -> bool {
		matches!(self, ContentPage::Overview(_))
	}

	pub fn project_page_mut(&mut self) -> Option<&mut ProjectPage> {
		if let ContentPage::Project(project_page) = self {
			Some(project_page)
		}
		else {
			None
		}
	}
}

impl ContentPage {
	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		match self {
			ContentPage::Project(project_page) => project_page.view(app),
			ContentPage::Overview(overview_page) => overview_page.view(app),
		}
	}
}