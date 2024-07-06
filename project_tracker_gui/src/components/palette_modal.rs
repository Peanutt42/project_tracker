use iced::{theme, widget::{button, column, container, row, scrollable, scrollable::RelativeOffset, text, text_input, Column}, Command, Element, Length, Padding};
use iced_aw::{ModalStyles, core::{Bootstrap, icons::bootstrap::icon_to_text}};
use once_cell::sync::Lazy;
use crate::{core::{Database, OrderedHashMap, Project, ProjectId}, project_tracker::UiMessage, styles::{scrollable_vertical_direction, PaletteContainerStyle, PaletteItemButtonStyle, PaletteModalStyle, ScrollableStyle, PADDING_AMOUNT, SCROLLBAR_WIDTH, SMALL_PADDING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT}};

static PALETTE_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
static PALETTE_SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

#[derive(Debug, Clone)]
pub enum PaletteItem {
	Overview,
	Settings,
	Project{ id: ProjectId, name: String },
}

impl PaletteItem {
	pub fn matches(&self, filter: &str) -> bool {
		match self {
			PaletteItem::Overview => "overview".contains(&filter.to_lowercase()),
			PaletteItem::Settings => "settings".contains(&filter.to_lowercase()),
			PaletteItem::Project{ name, .. } => name.to_lowercase().contains(&filter.to_lowercase()),
		}
	}

	pub fn view(&self, selected: bool) -> Element<'static, UiMessage> {
		button(
			match self {
				PaletteItem::Overview => row![
					icon_to_text(Bootstrap::List),
					text("Overview")
				]
				.spacing(SMALL_SPACING_AMOUNT)
				.into(),

				PaletteItem::Settings => row![
					icon_to_text(Bootstrap::Gear),
					text("Settings")
				]
				.spacing(SMALL_SPACING_AMOUNT)
				.into(),

				PaletteItem::Project { name, .. } => Element::new(text(name)),
			}
		)
		.style(theme::Button::custom(PaletteItemButtonStyle{ selected }))
		.on_press(PaletteModalMessage::OpenItem(self.clone()).into())
		.width(Length::Fill)
		.into()
	}
}

pub enum PaletteModal {
	Opened {
		input: String,
		selection_index: usize,
	},
	Closed,
}

impl PaletteModal {
	fn get_items_len(input: &str, projects: &OrderedHashMap<ProjectId, Project>) -> usize {
		let input = input.to_lowercase();
		let mut items_len = 0;

		if "overview".contains(&input) {
			items_len += 1;
		}
		if "settings".contains(&input) {
			items_len += 1;
		}
		for project in projects.values() {
			if project.name.to_lowercase().contains(&input) {
				items_len += 1;
			}
		}

		items_len
	}

	fn get_items(input: &str, projects: &OrderedHashMap<ProjectId, Project>) -> Vec<PaletteItem> {
		let input = input.to_lowercase();
		let mut items = Vec::new();

		if "overview".contains(&input) {
			items.push(PaletteItem::Overview);
		}
		if "settings".contains(&input) {
			items.push(PaletteItem::Settings);
		}
		for (id, project) in projects.iter() {
			if project.name.to_lowercase().contains(&input) {
				items.push(PaletteItem::Project{ id, name: project.name.clone() });
			}
		}

		items
	}

	pub fn get_selected_item(&self, database: &Option<Database>) -> Option<PaletteItem> {
		match self {
			PaletteModal::Opened { input, selection_index } => {
				database.as_ref().and_then(|database| {
					Self::get_items(input, &database.projects).get(*selection_index).cloned()
				})
			},
			PaletteModal::Closed => None,
		}
	}

	fn snap_to_selection(&mut self, items_len: usize) -> Command<UiMessage> {
		if let PaletteModal::Opened { selection_index, .. } = self {
			scrollable::snap_to(
				PALETTE_SCROLLABLE_ID.clone(),
				RelativeOffset{
					x: 0.0,
					y: *selection_index as f32 / (items_len as f32 - 1.0)
				}
			)
		}
		else {
			Command::none()
		}
	}

	pub fn update(&mut self, message: PaletteModalMessage, database: &Option<Database>) -> Command<UiMessage> {
		match message {
			PaletteModalMessage::Open => {
				*self = PaletteModal::Opened { input: String::new(), selection_index: 0 };
				text_input::focus(PALETTE_INPUT_ID.clone())
			},
			PaletteModalMessage::Close => { *self = PaletteModal::Closed; Command::none() },
			PaletteModalMessage::ToggleOpened => {
				match self {
					PaletteModal::Opened { .. } => self.update(PaletteModalMessage::Close, database),
					PaletteModal::Closed => self.update(PaletteModalMessage::Open, database),
				}
			},
			PaletteModalMessage::ChangeInput(new_input) => {
				if let PaletteModal::Opened { input, selection_index } = self {
					*input = new_input;
					if let Some(database) = database {
						let items_len = Self::get_items_len(input, &database.projects);
						if *selection_index >= items_len {
							*selection_index = items_len - 1;
						}
					}
				}
				Command::none()
			},
			PaletteModalMessage::SelectionUp => {
				if let PaletteModal::Opened { selection_index, input } = self {
					if *selection_index > 0 {
						*selection_index -= 1;
						if let Some(database) = database {
							let items_len = Self::get_items_len(input, &database.projects);
							return self.snap_to_selection(items_len);
						}
					}
				}
				Command::none()
			},
			PaletteModalMessage::SelectionDown => {
				if let PaletteModal::Opened { selection_index, input } = self {
					if let Some(database) = database {
						let items_len = Self::get_items_len(input, &database.projects);
						if *selection_index + 1 < items_len {
							*selection_index += 1;
							return self.snap_to_selection(items_len);
						}
					}
				}
				Command::none()
			},
			PaletteModalMessage::OpenItem(_) |
			PaletteModalMessage::OpenSelectedItem => self.update(PaletteModalMessage::Close, database),
		}
	}

	pub fn view(&self, database: &Option<Database>) -> Option<(Element<UiMessage>, ModalStyles)> {
		match self {
			PaletteModal::Opened { input, selection_index } => {
				database.as_ref().map(|database| {
					let selection_index = *selection_index;

					let item_views = Self::get_items(input, &database.projects)
				        .iter()
						.enumerate()
						.map(|(i, item)| {
							item.view(selection_index == i)
						})
						.collect();

					(
						container(
							container(
								column![
									text_input("search anything...", input)
										.id(PALETTE_INPUT_ID.clone())
										.on_input(|new_input| PaletteModalMessage::ChangeInput(new_input).into())
										.on_submit(PaletteModalMessage::OpenSelectedItem.into()),

									scrollable(
										Column::from_vec(item_views)
											.width(Length::Fill)
											.padding(Padding{ right: SMALL_PADDING_AMOUNT + SCROLLBAR_WIDTH, ..Padding::ZERO })
									)
									.id(PALETTE_SCROLLABLE_ID.clone())
									.direction(scrollable_vertical_direction())
									.style(theme::Scrollable::custom(ScrollableStyle))
								]
								.spacing(SPACING_AMOUNT)
							)
							.max_height(300.0)
							.padding(Padding::new(PADDING_AMOUNT))
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
			PaletteModal::Closed => None,
		}
	}
}

#[derive(Debug, Clone)]
pub enum PaletteModalMessage {
	Open,
	Close,
	ToggleOpened,
	ChangeInput(String),
	SelectionUp,
	SelectionDown,
	OpenItem(PaletteItem),
	OpenSelectedItem,
}

impl From<PaletteModalMessage> for UiMessage {
	fn from(value: PaletteModalMessage) -> Self {
		UiMessage::PaletteModalMessage(value)
	}
}