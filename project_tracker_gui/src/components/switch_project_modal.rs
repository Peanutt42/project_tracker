use iced::{theme, widget::{button, container, scrollable, scrollable::RelativeOffset, text, Column, row}, Command, Element, Length, Padding};
use iced_aw::ModalStyles;
use once_cell::sync::Lazy;
use crate::{components::project_color_block, core::{Database, ProjectId}, project_tracker::UiMessage, styles::{scrollable_vertical_direction, PaletteContainerStyle, PaletteItemButtonStyle, PaletteModalStyle, ScrollableStyle, PADDING_AMOUNT, SCROLLBAR_WIDTH, SMALL_PADDING_AMOUNT, TINY_SPACING_AMOUNT}};

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

pub enum SwitchProjectModal {
	Opened,
	Closed,
}

impl SwitchProjectModal {
	pub fn snap_to_selection(&mut self, selection_index: usize, items_len: usize) -> Command<UiMessage> {
		if let SwitchProjectModal::Opened = self {
			scrollable::snap_to(
				SCROLLABLE_ID.clone(),
				RelativeOffset{
					x: 0.0,
					y: selection_index as f32 / (items_len as f32 - 1.0)
				}
			)
		}
		else {
			Command::none()
		}
	}

	pub fn update(&mut self, message: SwitchProjectModalMessage, database: &Option<Database>, selected_project_id: Option<ProjectId>) -> Command<UiMessage> {
		match message {
			SwitchProjectModalMessage::Open => {
				*self = SwitchProjectModal::Opened;
				if let Some(database) = &database {
					if let Some(selected_project_id) = selected_project_id {
						if let Some(selected_project_order) = database.projects.get_order(&selected_project_id) {
							return self.snap_to_selection(selected_project_order, database.projects.len());
						}
					}
				}
				Command::none()
			},
			SwitchProjectModalMessage::Close => { *self = SwitchProjectModal::Closed; Command::none() },
		}
	}

	pub fn view(&self, database: &Option<Database>, selected_project_id: Option<ProjectId>) -> Option<(Element<UiMessage>, ModalStyles)> {
		match self {
			SwitchProjectModal::Opened => {
				database.as_ref().map(|database| {
					let selection_index = selected_project_id
						.map(|selected_project_id| {
							database.projects.get_order(&selected_project_id).unwrap_or(0)
						})
						.unwrap_or(0);

					let item_views = database.projects
				        .iter()
						.enumerate()
						.map(|(i, (project_id, project))| {
							button(
								row![
									project_color_block(project.color.into(), 20.0),
									text(&project.name),
								]
								.spacing(TINY_SPACING_AMOUNT)
							)
							.style(theme::Button::custom(PaletteItemButtonStyle{ selected: selection_index == i }))
							.on_press(UiMessage::SelectProject(Some(project_id)))
							.width(Length::Fill)
							.into()
						})
						.collect();

					(
						container(
							container(
								scrollable(
									Column::from_vec(item_views)
										.width(Length::Fill)
										.padding(Padding{ right: SMALL_PADDING_AMOUNT + SCROLLBAR_WIDTH, ..Padding::ZERO })
								)
								.id(SCROLLABLE_ID.clone())
								.direction(scrollable_vertical_direction())
								.style(theme::Scrollable::custom(ScrollableStyle))
							)
							.max_height(300.0)
							.padding(Padding{ left: PADDING_AMOUNT, right: 0.0, top: PADDING_AMOUNT, bottom: PADDING_AMOUNT })
							.style(theme::Container::Custom(Box::new(PaletteContainerStyle)))
						)
						.width(Length::Fill)
						.max_width(500.0)
						.center_x()
						.height(Length::Fill)
						.padding(Padding{ top: 75.0, ..Padding::ZERO })
						.into(),

						ModalStyles::custom(PaletteModalStyle)
					)
				})
			},
			SwitchProjectModal::Closed => None,
		}
	}
}

#[derive(Debug, Clone)]
pub enum SwitchProjectModalMessage {
	Open,
	Close,
}

impl From<SwitchProjectModalMessage> for UiMessage {
	fn from(value: SwitchProjectModalMessage) -> Self {
		UiMessage::SwitchProjectModalMessage(value)
	}
}