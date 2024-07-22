mod sidebar_page;
pub use sidebar_page::{SidebarPage, SidebarPageMessage};

mod project_page;
pub use project_page::{ProjectPage, ProjectPageMessage};

mod overview_page;
pub use overview_page::OverviewPage;

use iced::Element;
use crate::project_tracker::{ProjectTrackerApp, UiMessage};

pub enum ContentPage {
	Project(ProjectPage),
	Overview(OverviewPage),
}

impl ContentPage {
	pub fn is_overview_page(&self) -> bool {
		matches!(self, ContentPage::Overview(_))
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