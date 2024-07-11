use iced::{alignment::Horizontal, theme, widget::{column, container, row, scrollable, scrollable::RelativeOffset, text_input, Column}, Alignment, Command, Color, Element, Length, Padding};
use once_cell::sync::Lazy;
use crate::{core::{Database, DatabaseMessage}, project_tracker::UiMessage, styles::SMALL_SPACING_AMOUNT};
use crate::components::{create_new_project_button, loading_screen, overview_button, partial_horizontal_seperator, project_preview, custom_project_preview, EDIT_PROJECT_NAME_TEXT_INPUT_ID, settings_button, toggle_sidebar_button};
use crate::styles::{TextInputStyle, ScrollableStyle, scrollable_vertical_direction, LARGE_TEXT_SIZE, SMALL_PADDING_AMOUNT, PADDING_AMOUNT, SCROLLBAR_WIDTH, SPACING_AMOUNT};
use crate::project_tracker::ProjectTrackerApp;
use crate::core::{OrderedHashMap, ProjectId, generate_project_id, Project};

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
static TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

#[derive(Clone, Debug)]
pub enum SidebarPageMessage {
	OpenCreateNewProject,
	CloseCreateNewProject,
	ChangeCreateNewProjectName(String),

	EditProject(ProjectId),
	StopEditingProject,
}

impl From<SidebarPageMessage> for UiMessage {
	fn from(value: SidebarPageMessage) -> Self {
		UiMessage::SidebarPageMessage(value)
	}
}

#[derive(Clone)]
pub struct SidebarPage {
	create_new_project_name: Option<String>,
	pub project_being_edited: Option<ProjectId>,
}

impl SidebarPage {
	pub fn new() -> Self {
		Self {
			create_new_project_name: None,
			project_being_edited: None,
		}
	}

	pub fn snap_to_project(&mut self, project_order: usize, database: &Database) -> Command<UiMessage> {
		scrollable::snap_to(
			SCROLLABLE_ID.clone(),
			RelativeOffset {
				x: 0.0,
				y: project_order as f32 / (database.projects.len() as f32 - 1.0),
			}
		)
	}

	fn project_preview_list<'a>(&'a self, projects: &'a OrderedHashMap<ProjectId, Project>, app: &'a ProjectTrackerApp) -> Element<'a, UiMessage> {
		let mut list: Vec<Element<UiMessage>> = projects.iter()
			.map(|(project_id, project)| {
				let selected = match app.selected_project_id {
					Some(selected_project_id) => project_id == selected_project_id,
					None => false,
				};
				project_preview(project, project_id, selected)
			})
			.collect();

		if let Some(create_new_project_name) = &self.create_new_project_name {
			let project_name_text_input_element = container(
				text_input("New project name", create_new_project_name)
					.id(TEXT_INPUT_ID.clone())
					.size(LARGE_TEXT_SIZE)
					.on_input(|input| SidebarPageMessage::ChangeCreateNewProjectName(input).into())
					.on_submit(DatabaseMessage::CreateProject{
						project_id: generate_project_id(),
						name: create_new_project_name.clone()
					}.into())
					.style(theme::TextInput::Custom(Box::new(TextInputStyle)))
			)
			.width(Length::Fill)
			.align_x(Horizontal::Center)
			.into();

			list.push(custom_project_preview(None, Color::BLACK, 0.0, 0, 0, project_name_text_input_element, true));
		}

		scrollable(
			Column::from_vec(list)
				.width(Length::Fill)
				.spacing(SPACING_AMOUNT)
				.padding(Padding{ right: SMALL_PADDING_AMOUNT + SCROLLBAR_WIDTH, ..Padding::ZERO })
		)
		.id(SCROLLABLE_ID.clone())
		.width(Length::Fill)
		.height(Length::Fill)
		.style(theme::Scrollable::custom(ScrollableStyle))
		.direction(scrollable_vertical_direction())
		.into()
	}

	pub fn update(&mut self, message: SidebarPageMessage) -> Command<UiMessage> {
		match message {
			SidebarPageMessage::OpenCreateNewProject => {
				self.create_new_project_name = Some(String::new());
				Command::batch([
					self.update(SidebarPageMessage::StopEditingProject),
					text_input::focus(TEXT_INPUT_ID.clone()),
					scrollable::snap_to(SCROLLABLE_ID.clone(), RelativeOffset::END),
				])
			},
			SidebarPageMessage::CloseCreateNewProject => { self.create_new_project_name = None; Command::none() },
			SidebarPageMessage::ChangeCreateNewProjectName(new_project_name) => { self.create_new_project_name = Some(new_project_name); Command::none() },

			SidebarPageMessage::EditProject(project_id) => {
				self.project_being_edited = Some(project_id);
				Command::batch([
					self.update(SidebarPageMessage::CloseCreateNewProject),
					text_input::focus(EDIT_PROJECT_NAME_TEXT_INPUT_ID.clone())
				])
			},
			SidebarPageMessage::StopEditingProject => {
				self.project_being_edited = None;
				Command::none()
			},
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		let scrollbar_padding = Padding{ right: SMALL_PADDING_AMOUNT + SCROLLBAR_WIDTH, ..Padding::ZERO };

		let list: Element<UiMessage> = if let Some(database) = &app.database {
				self.project_preview_list(&database.projects, app)
			}
			else {
				container(loading_screen())
					.padding(scrollbar_padding)
					.into()
			};

		column![
			container(
				column![
					row![
						overview_button(app.content_page.is_overview_page()),
						toggle_sidebar_button(),
					]
					.align_items(Alignment::Center)
					.spacing(SMALL_SPACING_AMOUNT),

					partial_horizontal_seperator(),

					container(create_new_project_button(self.create_new_project_name.is_none()))
						.width(Length::Fill)
						.align_x(Horizontal::Right),
				]
				.spacing(SPACING_AMOUNT)
			)
			.padding(scrollbar_padding),

			list,

			container(
				column![
					partial_horizontal_seperator(),
					settings_button(app.content_page.is_settings_page()),
				]
				.spacing(SPACING_AMOUNT)
			)
			.padding(scrollbar_padding),
		]
		.width(Length::Fill)
		.height(Length::Fill)
		.spacing(SPACING_AMOUNT)
		.padding(Padding{ left: PADDING_AMOUNT, right: 0.0, top: PADDING_AMOUNT, bottom: PADDING_AMOUNT })
		.into()
	}
}

impl Default for SidebarPage {
	fn default() -> Self {
		Self::new()
	}
}