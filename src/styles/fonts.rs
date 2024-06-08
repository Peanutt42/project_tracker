use iced::{font::{Family, Stretch, Style, Weight}, Font};

pub const BOLD_FONT: Font = Font{
	weight: Weight::Bold,
	family: Family::SansSerif,
	stretch: Stretch::Normal,
	style: Style::Normal
};

pub fn strikethrough_text(text: &str) -> String {
	let mut result = String::with_capacity(text.len() * 2);
	for char in text.chars() {
		result.push(char);
		result.push('\u{0336}');
	}
	result
}
