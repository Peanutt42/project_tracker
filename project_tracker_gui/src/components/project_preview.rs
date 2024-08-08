use iced::{alignment::Horizontal, theme, widget::{container, container::Id, row, text}, Alignment, Border, Color, Element, Length, Padding};
use iced_aw::{quad::Quad, widgets::InnerBounds};
use iced_drop::droppable;
use crate::{pages::SidebarPageMessage, project_tracker::UiMessage, styles::DROP_HIGHLIGHT_WIDTH};
use crate::styles::{ProjectPreviewButtonStyle, DropZoneContainerStyle, ProjectPreviewBackgroundContainerStyle, SMALL_TEXT_SIZE, LARGE_TEXT_SIZE, TINY_SPACING_AMOUNT, SMALL_SPACING_AMOUNT, SMALL_PADDING_AMOUNT};
use crate::core::{Project, ProjectId};
use crate::components::cancel_create_project_button;

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

pub fn project_preview(project: &Project, project_id: ProjectId, selected: bool, dropzone_highlight: bool, dragging: bool, just_minimal_dragging: bool) -> Element<UiMessage> {
	let inner_text_element = text(&project.name).size(LARGE_TEXT_SIZE).into();

	custom_project_preview(
		Some(project_id),
		Some(project.preview_dropzone_id.clone()),
		project.color.into(),
		project.get_tasks_done(),
		project.tasks.len(),
		inner_text_element,
		selected,
		dropzone_highlight,
		dragging,
		just_minimal_dragging
	)
}

#[allow(clippy::too_many_arguments)]
pub fn custom_project_preview(project_id: Option<ProjectId>, dropzone_id: Option<Id>, project_color: Color, tasks_done: usize, task_len: usize, inner_text_element: Element<UiMessage>, selected: bool, dropzone_highlight: bool, dragging: bool, just_minimal_dragging: bool) -> Element<UiMessage> {
	let inner = container(
		row![
			project_color_block(project_color, DEFAULT_PROJECT_COLOR_BLOCK_HEIGHT),

			row![
				inner_text_element,
			]
			.push_maybe(
				if project_id.is_some() {
					Some(
						container(
							text(format!("({}/{})", tasks_done, task_len))
								.size(SMALL_TEXT_SIZE)
						)
						.width(if project_id.is_some() { Length::Fill } else { Length::Shrink })
						.align_x(Horizontal::Right)
					)
				}
				else {
					None
				}
			)
			.width(Length::Fill)
			.spacing(SMALL_SPACING_AMOUNT)
		]
		.align_items(Alignment::Center)
		.spacing(TINY_SPACING_AMOUNT)
		.padding(Padding{ right: SMALL_PADDING_AMOUNT, ..Padding::ZERO })
	)
	.style(theme::Container::Custom(Box::new(ProjectPreviewBackgroundContainerStyle{ dragging })));

	let underlay =
		container(
			inner
		)
		.id(
			dropzone_id.unwrap_or(container::Id::unique())
		)
		.width(Length::Fill)
		.padding(Padding::new(DROP_HIGHLIGHT_WIDTH))
		.style(theme::Container::Custom(Box::new(DropZoneContainerStyle{ highlight: dropzone_highlight })));

	if let Some(project_id) = project_id {
		droppable(
			underlay
		)
		.on_drop(move |point, rect| SidebarPageMessage::DropProject { project_id, point, rect }.into())
		.on_drag(move |point, rect| SidebarPageMessage::DragProject { project_id, point, rect }.into())
		.on_click(SidebarPageMessage::ClickProject(project_id).into())
		.on_cancel(SidebarPageMessage::CancelDragProject.into())
		.drag_overlay(!just_minimal_dragging)
		.drag_hide(!just_minimal_dragging)
		.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected }))
		.into()
	}
	else {
		row![
			underlay,
			cancel_create_project_button()
		]
		.align_items(Alignment::Center)
		.width(Length::Fill)
   		.into()
	}
}
