pub fn strikethrough_text(text: &str) -> String {
	let mut result = String::with_capacity(text.len() * 2);
	for char in text.chars() {
		result.push(char);
		result.push('\u{0336}');
	}
	result
}
