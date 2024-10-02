use crate::{
	core::{OrderedHashMap, TaskTag, TaskTagId},
	project_tracker::UiMessage,
	styles::TINY_SPACING_AMOUNT,
};
use iced::{widget::Row, Element};
use std::collections::HashSet;

use super::task_tag_button;

pub fn task_tags_buttons<'a>(
	available_tags: &'a OrderedHashMap<TaskTagId, TaskTag>,
	tags: &'a HashSet<TaskTagId>,
	on_press: impl Fn(TaskTagId) -> UiMessage,
) -> Element<'a, UiMessage> {
	Row::with_children(available_tags.iter().map(|(tag_id, tag)| {
		let toggled = tags.contains(&tag_id);
		task_tag_button(tag, toggled, !toggled, false)
			.on_press(on_press(tag_id))
			.into()
	}))
	.spacing(TINY_SPACING_AMOUNT)
	.into()
}
