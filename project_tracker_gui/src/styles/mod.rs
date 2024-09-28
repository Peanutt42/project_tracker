mod button_styles;
pub use button_styles::{project_preview_style, hidden_secondary_button_style, dangerous_button_style, delete_done_tasks_button_style, delete_button_style, circle_button_style, primary_button_style, secondary_button_style, secondary_button_style_default, secondary_button_style_only_round_right, secondary_button_style_only_round_bottom, selection_list_button_style, color_palette_button_style, invisible_button_style, task_tag_button_style, task_button_style, settings_tab_button_style, timer_button_style};

mod container_styles;
pub use container_styles::{rounded_container_style, palette_container_style, project_preview_background_container_style, task_background_container_style, dropzone_container_style, in_between_dropzone_container_style, tooltip_container_style, shadow_container_style, task_tag_container_style};

mod completion_bar_style;
pub use completion_bar_style::completion_bar_style;

// TODO: pane_grid replacement
/*mod split_style;
pub use split_style::SplitStyle;*/

mod checkbox_style;
pub use checkbox_style::checkbox_style;

mod text_input;
pub use text_input::{text_input_style, text_input_style_default, text_input_style_only_round_left};

mod text_editor;
pub use text_editor::text_editor_style;

mod scrollable;
pub use scrollable::scrollable_style;

mod card_style;
pub use card_style::card_style;

mod constants;
pub use constants::{
	colors::{color_average, mix_color, background_shadow_color, GREY, SELECTION_COLOR},
	padding::{SMALL_HORIZONTAL_PADDING, SMALL_PADDING_AMOUNT, PADDING_AMOUNT, LARGE_PADDING_AMOUNT},
	size::{SMALL_TEXT_SIZE, LARGE_TEXT_SIZE, HEADING_TEXT_SIZE, TITLE_TEXT_SIZE, MINIMAL_DRAG_DISTANCE},
	spacing::{TINY_SPACING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT, LARGE_SPACING_AMOUNT},
	border_radius::{BORDER_RADIUS, LARGE_BORDER_RADIUS},
	blur_radius::{BLUR_RADIUS, LARGE_BLUR_RADIUS},
	GAP,
};

mod text;
pub use text::text_color;

mod theme;
pub use theme::ProjectTrackerTheme;
