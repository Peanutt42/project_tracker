use std::{
	cell::{Cell, RefCell},
	ops::Range,
	sync::Arc,
};

use crate::styles::{checkbox_style, markdown_code_container_style, JET_BRAINS_MONO_FONT};
use iced::{
	font, padding,
	widget::{checkbox, column, container, markdown, rich_text, row, scrollable, span, text},
	Color, Element, Font, Length, Pixels,
};

#[derive(Debug, Clone)]
pub enum MarkdownMessage {
	OpenUrl(String),
	ToggleCheckbox { checked: bool, range: Range<usize> },
}

// copied from iced_widget-0.13.4/src/markdown.rs:66..84
// TODO: specify modifications
/// A Markdown item.
#[derive(Debug, Clone, PartialEq)]
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
		items: Arc<[ListItems]>,
	},
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListItems {
	beginner: ListItemBeginner,
	items: Arc<[Item]>,
}

impl ListItems {
	pub fn new(items: Arc<[Item]>) -> Self {
		Self {
			items,
			beginner: ListItemBeginner::default(),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
enum DynamicItem {
	Heading(pulldown_cmark::HeadingLevel, Text),
	Paragraph(Text),
	CodeBlock(Text),
	List {
		start: Option<u64>,
		items: Vec<DynamicListItems>,
	},
}

impl From<DynamicItem> for Item {
	fn from(value: DynamicItem) -> Self {
		match value {
			DynamicItem::List { start, items } => Item::List {
				start,
				items: items.into_iter().map(|di| di.into()).collect(),
			},
			DynamicItem::Heading(heading_level, text) => Item::Heading(heading_level, text),
			DynamicItem::CodeBlock(text) => Item::CodeBlock(text),
			DynamicItem::Paragraph(text) => Item::Paragraph(text),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
struct DynamicListItems {
	beginner: ListItemBeginner,
	items: Vec<DynamicItem>,
}
impl From<DynamicListItems> for ListItems {
	fn from(value: DynamicListItems) -> Self {
		Self {
			beginner: value.beginner,
			items: value.items.into_iter().map(|di| di.into()).collect(),
		}
	}
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ListItemBeginner {
	/// either unordered: "•" or ordered: "n." where n is the index
	#[default]
	Default,
	Checkbox {
		checked: bool,
		range: Range<usize>,
	},
}

// copied from iced_widget-0.13.4/src/markdown.rs:86..188
/// A bunch of parsed Markdown text.
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
	spans: Vec<Span>,
	last_style: Cell<Option<markdown::Style>>,
	last_styled_spans: RefCell<Arc<[text::Span<'static, MarkdownMessage>]>>,
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
	pub fn spans(&self, style: markdown::Style) -> Arc<[text::Span<'static, MarkdownMessage>]> {
		if Some(style) != self.last_style.get() {
			*self.last_styled_spans.borrow_mut() =
				self.spans.iter().map(|span| span.view(&style)).collect();

			self.last_style.set(Some(style));
		}

		self.last_styled_spans.borrow().clone()
	}
}

#[derive(Debug, Clone, PartialEq)]
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
	fn view(&self, style: &markdown::Style) -> text::Span<'static, MarkdownMessage> {
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
					span.color(style.link_color)
						.link(MarkdownMessage::OpenUrl(link.to_string()))
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

fn markdown_task_checkbox<'a>(checked: bool, range: Range<usize>) -> Element<'a, MarkdownMessage> {
	checkbox("", checked)
		.style(checkbox_style)
		.on_toggle(move |new_checked| MarkdownMessage::ToggleCheckbox {
			checked: new_checked,
			range: range.clone(),
		})
		.into()
}

// copied from iced_widget-0.13.4/src/markdown.rs:616..702
// modification: font to JetBrainsMono (matching the task description text editor font)
pub fn markdown_with_jetbrainsmono_font<'a>(
	items: Arc<[Item]>,
	settings: markdown::Settings,
	style: markdown::Style,
) -> Element<'a, MarkdownMessage> {
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

	let blocks = items.iter().enumerate().map(|(i, item)| match item {
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
					ListItemBeginner::Checkbox { checked, range } =>
						markdown_task_checkbox(*checked, range.clone()),
				},
				markdown_with_jetbrainsmono_font(items.items.clone(), settings, style)
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
					ListItemBeginner::Checkbox { checked, range } =>
						markdown_task_checkbox(*checked, range.clone()),
				},
				markdown_with_jetbrainsmono_font(items.items.clone(), settings, style)
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

pub fn parse_markdown(markdown: &str) -> Arc<[Item]> {
	advanced_parse(markdown).map(|item| item.into()).collect()
}

// copied from iced_widget-0.13.4/src/markdown.rs:190..491
// TODO: add gfm support with blocknotes --> see 'pulldown_cmark::Options::ENABLE_GFM'
/// Parse the given Markdown content.
#[allow(clippy::expect_used, clippy::unwrap_used)]
fn advanced_parse(markdown: &str) -> impl Iterator<Item = DynamicItem> + '_ {
	struct List {
		start: Option<u64>,
		items: Vec<DynamicListItems>,
	}

	let mut spans = Vec::new();
	let mut strong = false;
	let mut emphasis = false;
	let mut strikethrough = false;
	let mut metadata = false;
	let mut table = false;
	let mut link = None;
	let mut lists: Vec<List> = Vec::new();
	let mut task_hint: bool = false;

	let mut highlighter = None;

	let parser = pulldown_cmark::Parser::new_ext(
		markdown,
		pulldown_cmark::Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
			| pulldown_cmark::Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
			| pulldown_cmark::Options::ENABLE_TABLES
			| pulldown_cmark::Options::ENABLE_STRIKETHROUGH
			| pulldown_cmark::Options::ENABLE_TASKLISTS,
	);

	let produce = |lists: &mut Vec<List>, item: DynamicItem| {
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
	parser
		.into_offset_iter()
		.filter_map(move |(event, range)| match event {
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
					let output = if spans.is_empty() {
						None
					} else {
						produce(
							&mut lists,
							DynamicItem::Paragraph(Text::new(spans.drain(..).collect())),
						)
					};

					lists.push(List {
						start: first_item,
						items: Vec::new(),
					});

					output
				}
				pulldown_cmark::Tag::Item => {
					lists
						.last_mut()
						.expect("list context")
						.items
						.push(DynamicListItems {
							beginner: ListItemBeginner::default(),
							items: Vec::new(),
						});
					None
				}
				pulldown_cmark::Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(
					_language,
				)) if !metadata && !table => {
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
					DynamicItem::Heading(level, Text::new(spans.drain(..).collect())),
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
				pulldown_cmark::TagEnd::Paragraph if !metadata && !table => {
					let spans: Vec<_> = spans.drain(..).collect();
					if spans.is_empty() {
						None
					} else {
						produce(&mut lists, DynamicItem::Paragraph(Text::new(spans)))
					}
				}
				pulldown_cmark::TagEnd::Item if !metadata && !table => {
					if spans.is_empty() {
						None
					} else {
						produce(
							&mut lists,
							DynamicItem::Paragraph(Text::new(spans.drain(..).collect())),
						)
					}
				}
				pulldown_cmark::TagEnd::List(_) if !metadata && !table => {
					let list = lists.pop().expect("list context");

					produce(
						&mut lists,
						DynamicItem::List {
							start: list.start,
							items: list.items,
						},
					)
				}
				pulldown_cmark::TagEnd::CodeBlock if !metadata && !table => {
					highlighter = None;

					produce(
						&mut lists,
						DynamicItem::CodeBlock(Text::new(spans.drain(..).collect())),
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

				match task_hint {
					true => {
						task_hint = false;
						produce(
							&mut lists,
							DynamicItem::Paragraph(Text::new(spans.drain(..).collect())),
						)
					}
					false => None,
				}
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
						last_item_list.beginner = ListItemBeginner::Checkbox {
							checked,
							range: range.clone(),
						};
					}
				}
				task_hint = true;

				None
			}
			_ => None,
		})
}

#[cfg(test)]
mod tests {
	use super::{advanced_parse, Span, Text};
	use crate::components::markdown::{DynamicItem, DynamicListItems, ListItemBeginner};
	use std::{
		cell::{Cell, RefCell},
		ops::Range,
		sync::Arc,
	};

	fn paragraph(text: &'static str) -> DynamicItem {
		DynamicItem::Paragraph(Text::new(vec![Span::Standard {
			text: text.to_string(),
			strikethrough: false,
			link: None,
			strong: false,
			emphasis: false,
			code: false,
		}]))
	}

	#[test]
	fn test_heading_markdown_parsing() {
		let markdown_str = "# Heading";
		let markdown_items: Vec<DynamicItem> = advanced_parse(markdown_str).collect();
		assert_eq!(
			markdown_items,
			vec![DynamicItem::Heading(
				pulldown_cmark::HeadingLevel::H1,
				Text {
					spans: vec![Span::Standard {
						text: "Heading".to_string(),
						strikethrough: false,
						link: None,
						strong: false,
						emphasis: false,
						code: false
					}],
					last_style: Cell::new(None),
					last_styled_spans: RefCell::new(Arc::new([]))
				}
			)]
		)
	}

	#[test]
	fn test_nested_list_markdown_parsing() {
		let markdown_str = r"
- A
	- B
		- C";

		let markdown_items: Vec<DynamicItem> = advanced_parse(markdown_str).collect();

		assert_eq!(
			markdown_items,
			vec![DynamicItem::List {
				start: None,
				items: vec![DynamicListItems {
					beginner: ListItemBeginner::Default,
					items: vec![
						paragraph("A"),
						DynamicItem::List {
							start: None,
							items: vec![DynamicListItems {
								beginner: ListItemBeginner::Default,
								items: vec![
									paragraph("B"),
									DynamicItem::List {
										start: None,
										items: vec![DynamicListItems {
											beginner: ListItemBeginner::Default,
											items: vec![paragraph("C")]
										}]
									}
								]
							}]
						}
					]
				}]
			}]
		)
	}

	#[test]
	fn test_task_checkbox_markdown_parsing() {
		let markdown_str = r"
- [X] task1
- [X] task2
- [ ] task3";
		let markdown_items: Vec<DynamicItem> = advanced_parse(markdown_str).collect();

		assert_eq!(
			markdown_items,
			vec![DynamicItem::List {
				start: None,
				items: vec![
					DynamicListItems {
						beginner: ListItemBeginner::Checkbox {
							checked: true,
							range: Range { start: 3, end: 6 }
						},
						items: vec![paragraph("task1")]
					},
					DynamicListItems {
						beginner: ListItemBeginner::Checkbox {
							checked: true,
							range: Range { start: 15, end: 18 }
						},
						items: vec![paragraph("task2")]
					},
					DynamicListItems {
						beginner: ListItemBeginner::Checkbox {
							checked: false,
							range: Range { start: 27, end: 30 }
						},
						items: vec![paragraph("task3")]
					}
				]
			}]
		)
	}
}
