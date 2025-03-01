use std::sync::Arc;

use crate::{
	components::markdown::{markdown_with_jetbrainsmono_font, Item, MarkdownMessage},
	components::on_input,
	project_tracker::Message,
	styles::{
		description_text_editor_style, markdown_background_container_style, markdown_style,
		text_editor_keybindings, JET_BRAINS_MONO_FONT, PADDING_AMOUNT,
	},
	ProjectTrackerApp,
};
use iced::{
	highlighter,
	widget::{
		container, markdown, text, text_editor,
		text_editor::{Action, Content},
	},
	Element,
	Length::Fill,
	Pixels,
};
use project_tracker_core::{DatabaseMessage, ProjectId, TaskId};

pub fn task_description(
	project_id: ProjectId,
	task_id: TaskId,
	task_description_markdown_items: Option<Arc<[Item]>>,
	app: &ProjectTrackerApp,
) -> Element<Message> {
	container(match task_description_markdown_items {
		Some(task_description_markdown_items) if !task_description_markdown_items.is_empty() => {
			markdown_with_jetbrainsmono_font(
				task_description_markdown_items,
				markdown::Settings {
					// default text_size = 16.0, Settings::default() sets this to text_size * 0.75
					code_size: Pixels(16.0 * 0.85),
					..Default::default()
				},
				markdown_style(app),
			)
			.map(move |markdown_message| match markdown_message {
				MarkdownMessage::OpenUrl(url) => Message::OpenUrl(url),
				MarkdownMessage::ToggleCheckbox { checked, range } => {
					DatabaseMessage::ToggleTaskDescriptionMarkdownTask {
						project_id,
						task_id,
						checked,
						range,
					}
					.into()
				}
			})
		}
		_ => text("No description").width(Fill).into(),
	})
	.padding(PADDING_AMOUNT)
	.style(markdown_background_container_style)
	.into()
}

pub fn task_description_editor<'a, Message: 'a + Clone>(
	task_description_content: &'a Content,
	on_action: impl Fn(Action) -> Message + 'a,
	on_exit_editor: Option<Message>,
	unindent_message: Message,
) -> Element<'a, Message> {
	let text_editor = text_editor(task_description_content)
		.on_action(on_action)
		.wrapping(text::Wrapping::Word)
		.font(JET_BRAINS_MONO_FONT)
		.highlight("markdown", highlighter::Theme::Base16Eighties)
		.style(description_text_editor_style)
		.key_binding(move |key_press| text_editor_keybindings(key_press, unindent_message.clone()));

	match on_exit_editor {
		Some(on_exit_editor) => on_input(text_editor).on_esc(on_exit_editor).into(),
		None => text_editor.into(),
	}
}
