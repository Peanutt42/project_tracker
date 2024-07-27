use iced::{alignment::Horizontal, theme, widget::{button, container, container::Id, row, text, text_input, Space}, Alignment, Border, Color, Element, Length, Padding};
use iced_aw::{quad::Quad, widgets::InnerBounds};
use once_cell::sync::Lazy;
use crate::{project_tracker::UiMessage, styles::DROP_HIGHLIGHT_WIDTH};
use crate::styles::{ProjectPreviewButtonStyle, DropZoneContainerStyle, SMALL_TEXT_SIZE, LARGE_TEXT_SIZE, TINY_SPACING_AMOUNT, SMALL_SPACING_AMOUNT, SMALL_PADDING_AMOUNT};
use crate::core::{Project, ProjectId};
use crate::components::cancel_create_project_button;

pub static EDIT_PROJECT_NAME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub const PROJECT_COLOR_BLOCK_WIDTH: f32 = 5.0;
const DEFAULT_PROJECT_COLOR_BLOCK_HEIGHT: f32 = 35.0;

pub fn project_color_block(color: Color, height: f32) -> Element<'static, UiMessage> {
	Quad {
		width: Length::Fixed(PROJECT_COLOR_BLOCK_WIDTH),
		height: Length::Fixed(height),
		inner_bounds: InnerBounds::Ratio(1.0, 1.0),
		quad_color: color.into(),
		quad_border: Border::with_radius(f32::MAX),
		..Default::default()
	}
	.into()
}

pub fn project_preview(project: &Project, project_id: ProjectId, selected: bool, task_hovering: bool) -> Element<UiMessage> {
	let inner_text_element = text(&project.name).size(LARGE_TEXT_SIZE).into();

	custom_project_preview(
		Some(project_id),
		Some(project.preview_container_id.clone()),
		project.color.into(),
		project.get_tasks_done(),
		project.tasks.len(),
		inner_text_element,
		selected,
		task_hovering
	)
}

#[allow(clippy::too_many_arguments)]
pub fn custom_project_preview(project_id: Option<ProjectId>, container_id: Option<Id>, project_color: Color, tasks_done: usize, task_len: usize, inner_text_element: Element<UiMessage>, selected: bool, task_hovering: bool) -> Element<UiMessage> {
	let project_color_block: Element<UiMessage> = if selected {
			Space::new(PROJECT_COLOR_BLOCK_WIDTH, DEFAULT_PROJECT_COLOR_BLOCK_HEIGHT)
				.into()
		}
		else {
			project_color_block(project_color, DEFAULT_PROJECT_COLOR_BLOCK_HEIGHT)
		};

	let inner = row![
		project_color_block,

		row![
			inner_text_element,
			container(
				text(format!("({}/{})", tasks_done, task_len))
					.size(SMALL_TEXT_SIZE)
			)
			.width(if project_id.is_some() { Length::Fill } else { Length::Shrink })
			.align_x(Horizontal::Right),
		]
		.width(Length::Fill)
		.spacing(SMALL_SPACING_AMOUNT)
	]
	.align_items(Alignment::Center)
	.spacing(TINY_SPACING_AMOUNT)
	.padding(Padding{ right: SMALL_PADDING_AMOUNT, ..Padding::ZERO });

	let underlay =
		container(
			button(inner)
				.width(Length::Fill)
				.on_press(UiMessage::SelectProject(project_id))
				.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected, color: Some(project_color) }))
		)
		.id(
			container_id.unwrap_or(container::Id::unique())
		)
		.width(Length::Fill)
		.padding(Padding::new(DROP_HIGHLIGHT_WIDTH))
		.style(theme::Container::Custom(Box::new(DropZoneContainerStyle{ hovered: task_hovering })));

	if project_id.is_some() {
		underlay.into()
	}
	else {
		row![
			underlay,
			cancel_create_project_button()
		]
		.align_items(Alignment::Center)
		.spacing(SMALL_SPACING_AMOUNT)
		.width(Length::Fill)
   		.into()
	}
}
