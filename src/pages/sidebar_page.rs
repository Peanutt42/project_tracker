use iced::{alignment::{Alignment, Horizontal, Vertical}, theme, widget::{column, container, row, scrollable, scrollable::RelativeOffset, text_input, Column}, Command, Element, Length, Padding};
use once_cell::sync::Lazy;
use crate::{project_tracker::UiMessage, styles::LARGE_TEXT_SIZE};
use crate::components::{cancel_button, custom_project_preview, create_new_project_button, loading_screen, overview_button, partial_horizontal_seperator, project_preview, settings_button};
use crate::styles::{TextInputStyle, HORIZONTAL_PADDING, SMALL_PADDING_AMOUNT, SPACING_AMOUNT};
use crate::project_tracker::ProjectTrackerApp;
use crate::project::Project;

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
static TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Debug, Clone)]
pub enum SidebarPageMessage {
	OpenCreateNewProject,
	CloseCreateNewProject,
	ChangeCreateNewProjectName(String),
}

impl From<SidebarPageMessage> for UiMessage {
	fn from(value: SidebarPageMessage) -> Self {
		UiMessage::SidebarPageMessage(value)
	}
}

#[derive(Debug, Clone)]
pub struct SidebarPage {
	create_new_project_name: Option<String>,
}

impl SidebarPage {
	pub fn new() -> Self {
		Self {
			create_new_project_name: None,
		}
	}

	fn project_preview_list<'a>(&self, projects: &'a [Project], app: &'a ProjectTrackerApp) -> Element<'a, UiMessage> {
		let mut list: Vec<Element<UiMessage>> = projects.iter()
			.map(|project| {
				let selected = project.name == app.selected_page_name;
				project_preview(project, selected)
			})
			.collect();

		let create_new_project_element = if let Some(create_new_project_name) = &self.create_new_project_name {
			let project_name_text_input_element = container(
				row![
					text_input("New project name", create_new_project_name)
						.id(TEXT_INPUT_ID.clone())
						.size(LARGE_TEXT_SIZE)
						.on_input(|input| SidebarPageMessage::ChangeCreateNewProjectName(input).into())
						.on_submit(UiMessage::CreateProject(create_new_project_name.clone()))
						.style(theme::TextInput::Custom(Box::new(TextInputStyle))),
	
					cancel_button()
						.on_press(SidebarPageMessage::CloseCreateNewProject.into())
				]
				.align_items(Alignment::Center)
			)
			.width(Length::Fill)
			.align_x(Horizontal::Center)
			.into();

			custom_project_preview(None, 0.0, 0, 0, project_name_text_input_element, true)
		}
		else {
			column![].into()
		};

		list.push(create_new_project_element);
		
		scrollable(
			Column::from_vec(list)
				.width(Length::Fill)
				.spacing(SPACING_AMOUNT)
		)
		.id(SCROLLABLE_ID.clone())
		.width(Length::Fill)
		.height(Length::Fill)
		.into()
	}

	pub fn update(&mut self, message: SidebarPageMessage) -> Command<UiMessage> {
		match message {
			SidebarPageMessage::OpenCreateNewProject => {
				self.create_new_project_name = Some(String::new());
				Command::batch([
					text_input::focus(TEXT_INPUT_ID.clone()),
					scrollable::snap_to(SCROLLABLE_ID.clone(), RelativeOffset::END),
				])
			},
			SidebarPageMessage::CloseCreateNewProject => { self.create_new_project_name = None; Command::none() },
			SidebarPageMessage::ChangeCreateNewProjectName(new_project_name) => { self.create_new_project_name = Some(new_project_name); Command::none() },
		}		
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		let list: Element<UiMessage> =
			if let Some(saved_state) = &app.saved_state {
				self.project_preview_list(&saved_state.projects, app)
			}
			else {
				loading_screen()
			};

		let create_new_project_button: Element<UiMessage> = if self.create_new_project_name.is_some() {
			column![].into()
		}
		else {
			container(create_new_project_button())
				.align_x(Horizontal::Center)
				.width(Length::Fill)
				.into()
		};

		column![
			column![
				overview_button(app.content_page.is_overview_page()),
				partial_horizontal_seperator(),

				list,
	
				partial_horizontal_seperator(),
	
				create_new_project_button,
			]
			.height(Length::Fill)
			.spacing(SPACING_AMOUNT)
			.padding(HORIZONTAL_PADDING),
			
			container(settings_button())
				.align_y(Vertical::Bottom)
		]
		.width(Length::Fill)
		.height(Length::Fill)
		.spacing(SPACING_AMOUNT)
		.padding(Padding { top: SMALL_PADDING_AMOUNT, ..Padding::ZERO })
		.into()
	}
}