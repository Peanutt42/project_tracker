use crate::{
	project_tracker::Message,
	styles::{
		description_text_editor_style, markdown_background_container_style, markdown_style,
		text_editor_keybindings, JET_BRAINS_MONO_FONT, PADDING_AMOUNT,
	},
	ProjectTrackerApp,
};
use iced::{
	highlighter, padding,
	widget::{
		column, container, markdown, rich_text, row, scrollable, text, text_editor,
		text_editor::{Action, Content},
		TextEditor,
	},
	Element, Font,
	Length::{self, Fill},
	Pixels,
};

pub fn generate_task_description_markdown(description: &str) -> Vec<markdown::Item> {
	markdown::parse(description).collect()
}

pub fn task_description<'a>(
	task_description_markdown_items: Option<&'a Vec<markdown::Item>>,
	app: &'a ProjectTrackerApp,
) -> Element<'a, Message> {
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
			.map(|markdown_url| Message::OpenUrl(markdown_url.to_string()))
		}
		_ => text("No description").width(Fill).into(),
	})
	.padding(PADDING_AMOUNT)
	.style(markdown_background_container_style)
	.into()
}

pub fn task_description_editor<'a>(
	task_description_content: &'a Content,
	on_action: impl Fn(Action) -> Message + 'a,
	unindent_message: Message,
) -> TextEditor<'a, highlighter::Highlighter, Message> {
	text_editor(task_description_content)
		.on_action(on_action)
		.wrapping(text::Wrapping::Word)
		.font(JET_BRAINS_MONO_FONT)
		.highlight("markdown", highlighter::Theme::Base16Eighties)
		.style(description_text_editor_style)
		.key_binding(move |key_press| text_editor_keybindings(key_press, unindent_message.clone()))
}

// copied from iced_widget-0.13.4/src/markdown.rs:616..702
// modification: font to JetBrainsMono
fn markdown_with_jetbrainsmono_font<'a, Theme, Renderer>(
	items: impl IntoIterator<Item = &'a markdown::Item>,
	settings: markdown::Settings,
	style: markdown::Style,
) -> Element<'a, markdown::Url, Theme, Renderer>
where
	Theme: markdown::Catalog + 'a,
	Renderer: iced::advanced::text::Renderer<Font = Font> + 'a,
{
	let markdown::Settings {
		text_size,
		h1_size,
		h2_size,
		h3_size,
		h4_size,
		h5_size,
		h6_size,
		code_size,
	} = settings;

	let spacing = text_size * 0.625;

	let blocks = items.into_iter().enumerate().map(|(i, item)| match item {
		markdown::Item::Heading(level, heading) => container(
			rich_text(heading.spans(style))
				.size(match level {
					pulldown_cmark::HeadingLevel::H1 => h1_size,
					pulldown_cmark::HeadingLevel::H2 => h2_size,
					pulldown_cmark::HeadingLevel::H3 => h3_size,
					pulldown_cmark::HeadingLevel::H4 => h4_size,
					pulldown_cmark::HeadingLevel::H5 => h5_size,
					pulldown_cmark::HeadingLevel::H6 => h6_size,
				})
				.font(JET_BRAINS_MONO_FONT),
		)
		.padding(padding::top(if i > 0 {
			text_size / 2.0
		} else {
			Pixels::ZERO
		}))
		.into(),
		markdown::Item::Paragraph(paragraph) => rich_text(paragraph.spans(style))
			.size(text_size)
			.font(JET_BRAINS_MONO_FONT)
			.into(),
		markdown::Item::List { start: None, items } => column(items.iter().map(|items| {
			row![
				text("•").font(JET_BRAINS_MONO_FONT).size(text_size),
				markdown_with_jetbrainsmono_font(items, settings, style)
			]
			.spacing(spacing)
			.into()
		}))
		.spacing(spacing)
		.into(),
		markdown::Item::List {
			start: Some(start),
			items,
		} => column(items.iter().enumerate().map(|(i, items)| {
			row![
				text!("{}.", i as u64 + *start)
					.size(text_size)
					.font(JET_BRAINS_MONO_FONT),
				markdown_with_jetbrainsmono_font(items, settings, style)
			]
			.spacing(spacing)
			.into()
		}))
		.spacing(spacing)
		.into(),
		markdown::Item::CodeBlock(code) => container(
			scrollable(
				container(
					rich_text(code.spans(style))
						.font(JET_BRAINS_MONO_FONT)
						.size(code_size),
				)
				.padding(spacing.0 / 2.0),
			)
			.direction(scrollable::Direction::Horizontal(
				scrollable::Scrollbar::default()
					.width(spacing.0 / 2.0)
					.scroller_width(spacing.0 / 2.0),
			)),
		)
		.width(Length::Fill)
		.padding(spacing.0 / 2.0)
		.class(Theme::code_block())
		.into(),
	});

	Element::new(column(blocks).width(Length::Fill).spacing(text_size))
}
