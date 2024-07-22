pub fn strikethrough_text(text: &str) -> String {
	let mut result = String::with_capacity(text.len() * 2);
	for char in text.chars() {
		result.push(char);
		result.push('\u{0336}');
	}
	result
}

pub fn capped_text(text: &str, max_chars: usize) -> String {
	if text.len() <= max_chars {
		text.to_string()
	}
	else {
		// removes 3 chars for '...', while not gettings negativ
		let remaining_chars = max_chars.saturating_sub(3);
		format!("{}...", &text[..remaining_chars])
	}
}