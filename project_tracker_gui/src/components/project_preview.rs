use iced::{alignment::Horizontal, theme, widget::{column, container, container::Id, row, text, Space}, Alignment, Border, Color, Element, Length, Padding};
use iced_aw::{quad::Quad, widgets::InnerBounds};
use iced_drop::droppable;
use crate::{pages::SidebarPageMessage, project_tracker::UiMessage, styles::DropzoneContainerStyle};
use crate::styles::{ProjectPreviewButtonStyle, ProjectPreviewBackgroundContainerStyle, SMALL_TEXT_SIZE, LARGE_TEXT_SIZE, TINY_SPACING_AMOUNT, SMALL_SPACING_AMOUNT, SMALL_PADDING_AMOUNT};
use crate::core::{Project, ProjectId};
use crate::components::{cancel_create_project_button, in_between_dropzone};

pub const PROJECT_COLOR_BLOCK_WIDTH: f32 = 5.0;
const PROJECT_COLOR_BLOCK_HEIGHT: f32 = 35.0;

fn project_color_block(color: Color) -> Element<'static, UiMessage> {
	Quad {
		width: Length::Fixed(PROJECT_COLOR_BLOCK_WIDTH),
		height: Length::Fixed(PROJECT_COLOR_BLOCK_HEIGHT),
		inner_bounds: InnerBounds::Ratio(1.0, 1.0),
		quad_color: color.into(),
		quad_border: Border::with_radius(f32::MAX),
		..Default::default()
	}
	.into()
}

pub fn project_preview(project: &Project, project_id: ProjectId, selected: bool, project_dropzone_highlight: bool, task_dropzone_highlight: bool, dragging: bool, just_minimal_dragging: bool) -> Element<UiMessage> {
	let inner_text_element = text(&project.name).size(LARGE_TEXT_SIZE).into();

	custom_project_preview(
		Some(project_id),
		Some(project.project_dropzone_id.clone()),
		Some(project.task_dropzone_id.clone()),
		project.color.into(),
		project.done_tasks.len(),
		project.total_tasks(),
		inner_text_element,
		selected,
		project_dropzone_highlight,
		task_dropzone_highlight,
		dragging,
		just_minimal_dragging
	)
}

#[allow(clippy::too_many_arguments)]
pub fn custom_project_preview(project_id: Option<ProjectId>, project_dropzone_id: Option<Id>, task_dropzone_id: Option<Id>, project_color: Color, tasks_done: usize, task_len: usize, inner_text_element: Element<UiMessage>, selected: bool, project_dropzone_highlight: bool, task_dropzone_highlight: bool, dragging: bool, just_minimal_dragging: bool) -> Element<UiMessage> {
	let inner = container(
		row![
			if selected {
				Space::new(PROJECT_COLOR_BLOCK_WIDTH, PROJECT_COLOR_BLOCK_HEIGHT).into()
			}
			else {
				project_color_block(project_color)
			},

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
						.width(Length::Fill)
						.align_x(Horizontal::Right)
						.padding(Padding{ right: SMALL_PADDING_AMOUNT, ..Padding::ZERO })
					)
				}
				else {
					None
				}
			)
			.width(Length::Fill)
			.spacing(SMALL_SPACING_AMOUNT)
			.align_items(Alignment::Center)
		]
		.align_items(Alignment::Center)
		.spacing(TINY_SPACING_AMOUNT)
		.padding(Padding{ right: SMALL_PADDING_AMOUNT, ..Padding::ZERO })
	)
	.style(theme::Container::Custom(Box::new(ProjectPreviewBackgroundContainerStyle{ dragging })));

	if let Some(project_id) = project_id {
		column![
			in_between_dropzone(project_dropzone_id.unwrap_or(container::Id::unique()), project_dropzone_highlight),

			droppable(
				container(
					inner
				)
				.id(task_dropzone_id.unwrap_or(container::Id::unique()))
				.style(theme::Container::Custom(Box::new(DropzoneContainerStyle { highlight: task_dropzone_highlight })))
			)
			.on_drop(move |point, rect| SidebarPageMessage::DropProject { project_id, point, rect }.into())
			.on_drag(move |point, rect| SidebarPageMessage::DragProject { project_id, point, rect }.into())
			.on_click(SidebarPageMessage::ClickProject(project_id).into())
			.on_cancel(SidebarPageMessage::CancelDragProject.into())
			.drag_overlay(!just_minimal_dragging)
			.drag_hide(!just_minimal_dragging)
			.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected, project_color: Some(project_color) })),
		]
		.into()
	}
	else {
		row![
			inner,
			cancel_create_project_button()
		]
		.align_items(Alignment::Center)
		.width(Length::Fill)
   		.into()
	}
}
