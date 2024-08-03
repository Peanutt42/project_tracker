use std::collections::BTreeSet;
use iced::{theme, widget::{button, text, Row}, Element};
use crate::{core::{OrderedHashMap, TaskTag, TaskTagId}, project_tracker::UiMessage, styles::{TaskTagButtonStyle, TINY_SPACING_AMOUNT}};

pub fn task_tags_buttons<'a>(available_tags: &'a OrderedHashMap<TaskTagId, TaskTag>, tags: &'a BTreeSet<TaskTagId>, on_press: impl Fn(TaskTagId) -> UiMessage) -> Element<'a, UiMessage> {
	Row::with_children(
		available_tags
			.iter()
			.map(|(tag_id, tag)| {
				button(
					text(&tag.name)
				)
				.on_press(on_press(tag_id))
				.style(theme::Button::custom(
					TaskTagButtonStyle {
						color: tag.color.into(),
						toggled: tags.contains(&tag_id)
					}
				))
				.into()
			})
	)
	.spacing(TINY_SPACING_AMOUNT)
	.into()
}