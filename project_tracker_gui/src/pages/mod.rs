mod sidebar_page;
pub use sidebar_page::{SidebarPage, SidebarPageMessage};

mod project_page;
pub use project_page::{ProjectPage, ProjectPageMessage};

mod overview_page;
pub use overview_page::OverviewPage;

mod settings_page;
pub use settings_page::SettingsPage;

use iced::Element;
use crate::project_tracker::{ProjectTrackerApp, UiMessage};

pub enum ContentPage {
	Project(ProjectPage),
	Overview(OverviewPage),
	Settings(SettingsPage),
}

impl ContentPage {
	pub fn is_overview_page(&self) -> bool {
		matches!(self, ContentPage::Overview(_))
	}

	pub fn is_settings_page(&self) -> bool {
		matches!(self, ContentPage::Settings(_))
	}
}

impl ContentPage {
	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		match self {
			ContentPage::Project(project_page) => project_page.view(app),
			ContentPage::Overview(overview_page) => overview_page.view(app),
			ContentPage::Settings(settings_page) => settings_page.view(app),
		}
	}
}