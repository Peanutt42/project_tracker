mod button_styles;
pub use button_styles::{
	circle_button_style, dangerous_button_style, delete_button_style,
	delete_done_tasks_button_style, enum_dropdown_button_style, hidden_secondary_button_style,
	overview_button_style, primary_button_style, project_preview_style, secondary_button_style,
	secondary_button_style_default, secondary_button_style_no_rounding,
	secondary_button_style_only_round_left, secondary_button_style_only_round_right,
	secondary_button_style_only_round_top, selection_list_button_style, settings_tab_button_style,
	stopwatch_page_button_style, task_button_style, task_tag_button_style, timer_button_style,
};

mod container_styles;
pub use container_styles::{
	command_background_container_style, default_background_container_style,
	dropdown_container_style, dropzone_container_style, in_between_dropzone_container_style,
	markdown_background_container_style, markdown_code_container_style,
	modal_background_container_style, palette_container_style,
	project_preview_background_container_style, rounded_container_style,
	sidebar_background_container_style, task_background_container_style, task_tag_container_style,
	tooltip_container_style,
};

mod completion_bar_style;
pub use completion_bar_style::completion_bar_style;

mod checkbox_style;
pub use checkbox_style::checkbox_style;

mod text_input;
pub use text_input::{
	on_number_input, text_input_style, text_input_style_borderless, text_input_style_default,
	text_input_style_only_round_left,
};

mod text_editor;
pub use text_editor::{description_text_editor_style, text_editor_keybindings, unindent_text};

mod scrollable;
pub use scrollable::{logs_scrollable_style, scrollable_style};

mod card_style;
pub use card_style::card_style;

mod constants;
pub use constants::{
	blur_radius::{BLUR_RADIUS, LARGE_BLUR_RADIUS},
	border_radius::{BORDER_RADIUS, LARGE_BORDER_RADIUS},
	colors::{
		background_shadow_color, color_average, link_color, mix_color, selection_color, GREY,
	},
	padding::{
		LARGE_PADDING_AMOUNT, PADDING_AMOUNT, SMALL_HORIZONTAL_PADDING, SMALL_PADDING_AMOUNT,
	},
	size::{
		HEADING_TEXT_SIZE, LARGE_TEXT_SIZE, MINIMAL_DRAG_DISTANCE, SMALL_TEXT_SIZE, TITLE_TEXT_SIZE,
	},
	spacing::{LARGE_SPACING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT, TINY_SPACING_AMOUNT},
	GAP,
};

mod text;
pub use text::{
	danger_text_style, default_text_style, grey_text_style, text_color, BOLD_FONT, INTER_FONT,
	INTER_FONT_BYTES, JET_BRAINS_MONO_FONT, JET_BRAINS_MONO_FONT_BYTES, MONOSPACE_FONT,
};

mod theme;
pub use theme::ProjectTrackerTheme;

mod markdown_style;
pub use markdown_style::markdown_style;
