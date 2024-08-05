mod button_styles;
pub use button_styles::{ProjectPreviewButtonStyle, HiddenSecondaryButtonStyle, DangerousButtonStyle, DeleteDoneTasksButtonStyle, DeleteButtonStyle, RoundedSecondaryButtonStyle, CancelButtonStyle, ThemeModeButtonStyle, PaletteItemButtonStyle, ColorPaletteButtonStyle, InvisibleButtonStyle, TaskTagButtonStyle};

mod container_styles;
pub use container_styles::{RoundedContainerStyle, PaletteContainerStyle, ProjectPreviewBackgroundContainerStyle, TaskBackgroundContainerStyle, DropZoneContainerStyle, DROP_HIGHLIGHT_WIDTH};

mod completion_bar_style;
pub use completion_bar_style::CompletionBarStyle;

mod split_style;
pub use split_style::SplitStyle;

mod checkbox_style;
pub use checkbox_style::GreenCheckboxStyle;

mod text_input;
pub use text_input::TextInputStyle;

mod text_editor;
pub use text_editor::TextEditorStyle;

mod scrollable;
pub use scrollable::{ScrollableStyle, scrollable_horizontal_direction, scrollable_vertical_direction, SCROLLBAR_WIDTH};

mod modal_style;
pub use modal_style::{ModalStyle, PaletteModalStyle, ModalCardStyle};

mod constants;
pub use constants::{
	colors::{color_average, mix_color, NICE_GREEN, LIGHT_DARK_GREEN, GREY, DARK_GREY},
	padding::{HORIZONTAL_PADDING, SMALL_HORIZONTAL_PADDING, SMALL_PADDING_AMOUNT, PADDING_AMOUNT, LARGE_PADDING_AMOUNT},
	size::{SMALL_TEXT_SIZE, LARGE_TEXT_SIZE, HEADING_TEXT_SIZE, TITLE_TEXT_SIZE},
	spacing::{TINY_SPACING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT, LARGE_SPACING_AMOUNT},
	border_radius::{BORDER_RADIUS, LARGE_BORDER_RADIUS},
};

mod text;
pub use text::{strikethrough_text, text_color, GREEN_TEXT_STYLE, DISABLED_GREEN_TEXT_STYLE};

mod theme;
pub use theme::ProjectTrackerTheme;
