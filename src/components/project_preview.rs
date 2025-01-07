use crate::components::{cancel_create_project_button, in_between_dropzone};
use crate::core::IcedColorConversion;
use crate::styles::{
	project_preview_background_container_style, LARGE_TEXT_SIZE, SMALL_PADDING_AMOUNT,
	SMALL_SPACING_AMOUNT, SMALL_TEXT_SIZE, TINY_SPACING_AMOUNT,
};
use crate::{
	pages::SidebarPageMessage,
	project_tracker::Message,
	styles::{dropzone_container_style, project_preview_style},
};
use iced::{
	alignment::Horizontal,
	border::rounded,
	widget::{column, container, container::Id, row, text, Space},
	Alignment, Color, Element,
	Length::{self, Fill},
	Padding,
};
use iced_aw::{quad::Quad, widgets::InnerBounds};
use iced_drop::droppable;
use project_tracker_core::{Project, ProjectId};

pub const PROJECT_COLOR_BLOCK_WIDTH: f32 = 5.0;
const PROJECT_COLOR_BLOCK_HEIGHT: f32 = 35.0;

pub fn project_color_block(color: Color) -> Element<'static, Message> {
	Quad {
		width: Length::Fixed(PROJECT_COLOR_BLOCK_WIDTH),
		height: Length::Fixed(PROJECT_COLOR_BLOCK_HEIGHT),
		inner_bounds: InnerBounds::Ratio(1.0, 1.0),
		quad_color: color.into(),
		quad_border: rounded(f32::MAX),
		..Default::default()
	}
	.into()
}

#[allow(clippy::too_many_arguments)]
pub fn project_preview(
	project: &Project,
	project_id: ProjectId,
	project_dropzone_id: Id,
	task_dropzone_id: Id,
	selected: bool,
	project_dropzone_highlight: bool,
	task_dropzone_highlight: bool,
	dragging: bool,
	just_minimal_dragging: bool,
) -> Element<Message> {
	let inner_text_element = text(&project.name).size(LARGE_TEXT_SIZE).into();

	custom_project_preview(
		Some(project_id),
		Some(project_dropzone_id),
		Some(task_dropzone_id),
		project.color.to_iced_color(),
		project.done_tasks.len(),
		project.total_tasks(),
		inner_text_element,
		selected,
		project_dropzone_highlight,
		task_dropzone_highlight,
		dragging,
		just_minimal_dragging,
	)
}

#[allow(clippy::too_many_arguments)]
pub fn custom_project_preview(
	project_id: Option<ProjectId>,
	project_dropzone_id: Option<Id>,
	task_dropzone_id: Option<Id>,
	project_color: Color,
	tasks_done: usize,
	task_len: usize,
	inner_text_element: Element<Message>,
	selected: bool,
	project_dropzone_highlight: bool,
	task_dropzone_highlight: bool,
	dragging: bool,
	just_minimal_dragging: bool,
) -> Element<Message> {
	let inner = container(
		row![
			if selected && project_id.is_some() {
				Space::new(PROJECT_COLOR_BLOCK_WIDTH, PROJECT_COLOR_BLOCK_HEIGHT).into()
			} else {
				project_color_block(project_color)
			},
			row![inner_text_element,]
				.push_maybe(if project_id.is_some() {
					Some(
						container(
							text(format!("({}/{})", tasks_done, task_len)).size(SMALL_TEXT_SIZE),
						)
						.width(Fill)
						.align_x(Horizontal::Right)
						.padding(Padding {
							right: SMALL_PADDING_AMOUNT,
							..Padding::ZERO
						}),
					)
				} else {
					None
				})
				.width(Fill)
				.spacing(SMALL_SPACING_AMOUNT)
				.align_y(Alignment::Center)
		]
		.align_y(Alignment::Center)
		.spacing(TINY_SPACING_AMOUNT)
		.padding(Padding {
			right: SMALL_PADDING_AMOUNT,
			..Padding::ZERO
		}),
	)
	.style(move |t| project_preview_background_container_style(t, dragging));

	match project_id {
		Some(project_id) => column![
			in_between_dropzone(
				project_dropzone_id.unwrap_or(container::Id::unique()),
				project_dropzone_highlight
			),
			droppable(
				container(inner)
					.id(task_dropzone_id.unwrap_or(container::Id::unique()))
					.style(move |t| dropzone_container_style(t, task_dropzone_highlight))
			)
			.on_drop(move |point, rect| SidebarPageMessage::DropProject {
				project_id,
				point,
				rect
			}
			.into())
			.on_drag(move |point, rect| SidebarPageMessage::DragProject {
				project_id,
				point,
				rect
			}
			.into())
			.on_click(SidebarPageMessage::ClickProject(project_id).into())
			.on_cancel(SidebarPageMessage::CancelDragProject.into())
			.drag_overlay(!just_minimal_dragging)
			.drag_hide(!just_minimal_dragging)
			.drag_mode(false, true)
			.style(move |t, s| project_preview_style(t, s, selected, Some(project_color))),
		]
		.into(),
		None => row![inner, cancel_create_project_button()]
			.align_y(Alignment::Center)
			.width(Fill)
			.into(),
	}
}
