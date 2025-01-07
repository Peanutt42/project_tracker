use std::{
	cell::{Cell, RefCell},
	sync::Arc,
};

use crate::styles::{checkbox_style, markdown_code_container_style, JET_BRAINS_MONO_FONT};
use iced::{
	font, padding,
	widget::{checkbox, column, container, markdown, rich_text, row, scrollable, span, text},
	Color, Element, Font, Length, Pixels,
};

// copied from iced_widget-0.13.4/src/markdown.rs:66..84
// TODO: specify modifications
/// A Markdown item.
#[derive(Debug, Clone)]
pub enum Item {
	/// A heading.
	Heading(pulldown_cmark::HeadingLevel, Text),
	/// A paragraph.
	Paragraph(Text),
	/// A code block.
	///
	/// You can enable the `highlighter` feature for syntax highligting.
	CodeBlock(Text),
	/// A list.
	List {
		start: Option<u64>,
		/// The items of the list.
		items: Vec<ListItems>,
	},
}

#[derive(Debug, Clone)]
pub struct ListItems {
	beginner: ListItemBeginner,
	items: Vec<Item>,
}

impl ListItems {
	pub fn new(items: Vec<Item>) -> Self {
		Self {
			items,
			beginner: ListItemBeginner::default(),
		}
	}
}

#[derive(Debug, Clone, Default)]
pub enum ListItemBeginner {
	/// either unordered: "•" or ordered: "n." where n is the index
	#[default]
	Default,
	Checkbox {
		checked: bool,
	},
}

// copied from iced_widget-0.13.4/src/markdown.rs:86..188
/// A bunch of parsed Markdown text.
#[derive(Debug, Clone)]
pub struct Text {
	spans: Vec<Span>,
	last_style: Cell<Option<markdown::Style>>,
	last_styled_spans: RefCell<Arc<[text::Span<'static, markdown::Url>]>>,
}

impl Text {
	fn new(spans: Vec<Span>) -> Self {
		Self {
			spans,
			last_style: Cell::default(),
			last_styled_spans: RefCell::default(),
		}
	}

	/// Returns the [`rich_text()`] spans ready to be used for the given style.
	///
	/// This method performs caching for you. It will only reallocate if the [`Style`]
	/// provided changes.
	pub fn spans(&self, style: markdown::Style) -> Arc<[text::Span<'static, markdown::Url>]> {
		if Some(style) != self.last_style.get() {
			*self.last_styled_spans.borrow_mut() =
				self.spans.iter().map(|span| span.view(&style)).collect();

			self.last_style.set(Some(style));
		}

		self.last_styled_spans.borrow().clone()
	}
}

#[derive(Debug, Clone)]
enum Span {
	Standard {
		text: String,
		strikethrough: bool,
		link: Option<markdown::Url>,
		strong: bool,
		emphasis: bool,
		code: bool,
	},
	Highlight {
		text: String,
		color: Option<Color>,
		font: Option<Font>,
	},
}

impl Span {
	fn view(&self, style: &markdown::Style) -> text::Span<'static, markdown::Url> {
		match self {
			Span::Standard {
				text,
				strikethrough,
				link,
				strong,
				emphasis,
				code,
			} => {
				let span = span(text.clone()).strikethrough(*strikethrough);

				let span = if *code {
					span.font(Font::MONOSPACE)
						.color(style.inline_code_color)
						.background(style.inline_code_highlight.background)
						.border(style.inline_code_highlight.border)
						.padding(style.inline_code_padding)
				} else if *strong || *emphasis {
					span.font(Font {
						weight: if *strong {
							font::Weight::Bold
						} else {
							font::Weight::Normal
						},
						style: if *emphasis {
							font::Style::Italic
						} else {
							font::Style::Normal
						},
						..Font::default()
					})
				} else {
					span
				};

				let span = if let Some(link) = link.as_ref() {
					span.color(style.link_color).link(link.clone())
				} else {
					span
				};

				span
			}
			Span::Highlight { text, color, font } => {
				span(text.clone()).color_maybe(*color).font_maybe(*font)
			}
		}
	}
}

fn markdown_task_checkbox<'a>(checked: bool) -> Element<'a, markdown::Url> {
	checkbox("", checked).style(checkbox_style).into()
}

// copied from iced_widget-0.13.4/src/markdown.rs:616..702
// modification: font to JetBrainsMono (matching the task description text editor font)
pub fn markdown_with_jetbrainsmono_font<'a>(
	items: impl IntoIterator<Item = &'a Item>,
	settings: markdown::Settings,
	style: markdown::Style,
) -> Element<'a, markdown::Url>
/*where
	Theme: markdown::Catalog + 'a,
	Renderer: iced::advanced::text::Renderer<Font = Font> + 'a,*/
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
		Item::Heading(level, heading) => container(
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
		Item::Paragraph(paragraph) => rich_text(paragraph.spans(style))
			.size(text_size)
			.font(JET_BRAINS_MONO_FONT)
			.into(),
		Item::List { start: None, items } => column(items.iter().map(|items| {
			row![
				match &items.beginner {
					ListItemBeginner::Default =>
						text("•").font(JET_BRAINS_MONO_FONT).size(text_size).into(),
					ListItemBeginner::Checkbox { checked } => markdown_task_checkbox(*checked),
				},
				markdown_with_jetbrainsmono_font(&items.items, settings, style)
			]
			.spacing(spacing)
			.into()
		}))
		.spacing(spacing)
		.into(),
		Item::List {
			start: Some(start),
			items,
		} => column(items.iter().enumerate().map(|(i, items)| {
			row![
				match &items.beginner {
					ListItemBeginner::Default => text!("{}.", i as u64 + *start)
						.size(text_size)
						.font(JET_BRAINS_MONO_FONT)
						.into(),
					ListItemBeginner::Checkbox { checked } => markdown_task_checkbox(*checked),
				},
				markdown_with_jetbrainsmono_font(&items.items, settings, style)
			]
			.spacing(spacing)
			.into()
		}))
		.spacing(spacing)
		.into(),
		Item::CodeBlock(code) => container(
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
		.style(markdown_code_container_style)
		.into(),
	});

	Element::new(column(blocks).width(Length::Fill).spacing(text_size))
}

// copied from iced_widget-0.13.4/src/markdown.rs:190..491
// TODO: add gfm support with blocknotes --> see 'pulldown_cmark::Options::ENABLE_GFM'
/// Parse the given Markdown content.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::markdown;
/// use iced::Theme;
///
/// struct State {
///    markdown: Vec<markdown::Item>,
/// }
///
/// enum Message {
///     LinkClicked(markdown::Url),
/// }
///
/// impl State {
///     pub fn new() -> Self {
///         Self {
///             markdown: markdown::parse("This is some **Markdown**!").collect(),
///         }
///     }
///
///     fn view(&self) -> Element<'_, Message> {
///         markdown::view(
///             &self.markdown,
///             markdown::Settings::default(),
///             markdown::Style::from_palette(Theme::TokyoNightStorm.palette()),
///         )
///         .map(Message::LinkClicked)
///         .into()
///     }
///
///     fn update(state: &mut State, message: Message) {
///         match message {
///             Message::LinkClicked(url) => {
///                 println!("The following url was clicked: {url}");
///             }
///         }
///     }
/// }
/// ```
#[allow(clippy::expect_used, clippy::unwrap_used)]
pub fn advanced_parse(markdown: &str) -> impl Iterator<Item = Item> + '_ {
	struct List {
		start: Option<u64>,
		items: Vec<ListItems>,
	}

	let mut spans = Vec::new();
	let mut strong = false;
	let mut emphasis = false;
	let mut strikethrough = false;
	let mut metadata = false;
	let mut table = false;
	let mut link = None;
	let mut lists = Vec::new();

	let mut highlighter = None;

	let parser = pulldown_cmark::Parser::new_ext(
		markdown,
		pulldown_cmark::Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
			| pulldown_cmark::Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
			| pulldown_cmark::Options::ENABLE_TABLES
			| pulldown_cmark::Options::ENABLE_STRIKETHROUGH
			| pulldown_cmark::Options::ENABLE_TASKLISTS,
	);

	let produce = |lists: &mut Vec<List>, item| {
		if lists.is_empty() {
			Some(item)
		} else {
			lists
				.last_mut()
				.expect("list context")
				.items
				.last_mut()
				.expect("item context")
				.items
				.push(item);

			None
		}
	};

	// We want to keep the `spans` capacity
	#[allow(clippy::drain_collect)]
	parser.filter_map(move |event| match event {
		pulldown_cmark::Event::Start(tag) => match tag {
			pulldown_cmark::Tag::Strong if !metadata && !table => {
				strong = true;
				None
			}
			pulldown_cmark::Tag::Emphasis if !metadata && !table => {
				emphasis = true;
				None
			}
			pulldown_cmark::Tag::Strikethrough if !metadata && !table => {
				strikethrough = true;
				None
			}
			pulldown_cmark::Tag::Link { dest_url, .. } if !metadata && !table => {
				match markdown::Url::parse(&dest_url) {
					Ok(url) if url.scheme() == "http" || url.scheme() == "https" => {
						link = Some(url);
					}
					_ => {}
				}

				None
			}
			pulldown_cmark::Tag::List(first_item) if !metadata && !table => {
				lists.push(List {
					start: first_item,
					items: Vec::new(),
				});

				None
			}
			pulldown_cmark::Tag::Item => {
				lists
					.last_mut()
					.expect("list context")
					.items
					.push(ListItems::new(Vec::new()));
				None
			}
			pulldown_cmark::Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(_language))
				if !metadata && !table =>
			{
				{
					use iced::highlighter::Highlighter;
					use text::Highlighter as _;

					highlighter = Some(Highlighter::new(&iced::highlighter::Settings {
						theme: iced::highlighter::Theme::Base16Ocean,
						token: _language.to_string(),
					}));
				}

				None
			}
			pulldown_cmark::Tag::MetadataBlock(_) => {
				metadata = true;
				None
			}
			pulldown_cmark::Tag::Table(_) => {
				table = true;
				None
			}
			_ => None,
		},
		pulldown_cmark::Event::End(tag) => match tag {
			pulldown_cmark::TagEnd::Heading(level) if !metadata && !table => produce(
				&mut lists,
				Item::Heading(level, Text::new(spans.drain(..).collect())),
			),
			pulldown_cmark::TagEnd::Strong if !metadata && !table => {
				strong = false;
				None
			}
			pulldown_cmark::TagEnd::Emphasis if !metadata && !table => {
				emphasis = false;
				None
			}
			pulldown_cmark::TagEnd::Strikethrough if !metadata && !table => {
				strikethrough = false;
				None
			}
			pulldown_cmark::TagEnd::Link if !metadata && !table => {
				link = None;
				None
			}
			pulldown_cmark::TagEnd::Paragraph if !metadata && !table => produce(
				&mut lists,
				Item::Paragraph(Text::new(spans.drain(..).collect())),
			),
			pulldown_cmark::TagEnd::Item if !metadata && !table => {
				if spans.is_empty() {
					None
				} else {
					produce(
						&mut lists,
						Item::Paragraph(Text::new(spans.drain(..).collect())),
					)
				}
			}
			pulldown_cmark::TagEnd::List(_) if !metadata && !table => {
				let list = lists.pop().expect("list context");

				produce(
					&mut lists,
					Item::List {
						start: list.start,
						items: list.items,
					},
				)
			}
			pulldown_cmark::TagEnd::CodeBlock if !metadata && !table => {
				highlighter = None;

				produce(
					&mut lists,
					Item::CodeBlock(Text::new(spans.drain(..).collect())),
				)
			}
			pulldown_cmark::TagEnd::MetadataBlock(_) => {
				metadata = false;
				None
			}
			pulldown_cmark::TagEnd::Table => {
				table = false;
				None
			}
			_ => None,
		},
		pulldown_cmark::Event::Text(text) if !metadata && !table => {
			if let Some(highlighter) = &mut highlighter {
				use text::Highlighter as _;

				for (range, highlight) in highlighter.highlight_line(text.as_ref()) {
					let span = Span::Highlight {
						text: text[range].to_owned(),
						color: highlight.color(),
						font: highlight.font(),
					};

					spans.push(span);
				}

				return None;
			}

			let span = Span::Standard {
				text: text.into_string(),
				strong,
				emphasis,
				strikethrough,
				link: link.clone(),
				code: false,
			};

			spans.push(span);

			None
		}
		pulldown_cmark::Event::Code(code) if !metadata && !table => {
			let span = Span::Standard {
				text: code.into_string(),
				strong,
				emphasis,
				strikethrough,
				link: link.clone(),
				code: true,
			};

			spans.push(span);
			None
		}
		pulldown_cmark::Event::SoftBreak if !metadata && !table => {
			spans.push(Span::Standard {
				text: String::from(" "),
				strikethrough,
				strong,
				emphasis,
				link: link.clone(),
				code: false,
			});
			None
		}
		pulldown_cmark::Event::HardBreak if !metadata && !table => {
			spans.push(Span::Standard {
				text: String::from("\n"),
				strikethrough,
				strong,
				emphasis,
				link: link.clone(),
				code: false,
			});
			None
		}
		pulldown_cmark::Event::TaskListMarker(checked) if !metadata && !table => {
			if let Some(last_list) = lists.last_mut() {
				if let Some(last_item_list) = last_list.items.last_mut() {
					last_item_list.beginner = ListItemBeginner::Checkbox { checked };
				}
			}

			None
		}
		_ => None,
	})
}
