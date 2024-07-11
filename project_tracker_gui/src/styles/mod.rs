mod button_styles;
pub use button_styles::{ProjectPreviewButtonStyle, HiddenSecondaryButtonStyle, DangerousButtonStyle, DeleteDoneTasksButtonStyle, DeleteButtonStyle, RoundedSecondaryButtonStyle, ProjectContextButtonStyle, ThemeModeButtonStyle, PaletteItemButtonStyle, ColorPaletteButtonStyle};

mod container_styles;
pub use container_styles::{RoundedContainerStyle, PaletteContainerStyle};

mod completion_bar_style;
pub use completion_bar_style::CompletionBarStyle;

mod split_style;
pub use split_style::SplitStyle;

mod text_styles;
pub use text_styles::{GREEN_TEXT_STYLE, DISABLED_GREEN_TEXT_STYLE};

mod checkbox_style;
pub use checkbox_style::GreenCheckboxStyle;

mod text_input;
pub use text_input::TextInputStyle;

mod scrollable;
pub use scrollable::{ScrollableStyle, scrollable_vertical_direction, SCROLLBAR_WIDTH};

mod modal_style;
pub use modal_style::{ModalStyle, PaletteModalStyle, ModalCardStyle};

mod constants;
pub use constants::{
	colors::{mix_color, NICE_GREEN, LIGHT_DARK_GREEN, LIGHT_GREY, GREY, DARK_GREY},
	padding::{HORIZONTAL_PADDING, SMALL_HORIZONTAL_PADDING, SMALL_PADDING_AMOUNT, PADDING_AMOUNT, LARGE_PADDING_AMOUNT},
	size::{SMALL_TEXT_SIZE, MIDDLE_TEXT_SIZE, LARGE_TEXT_SIZE, TITLE_TEXT_SIZE},
	spacing::{TINY_SPACING_AMOUNT, SMALL_SPACING_AMOUNT, SPACING_AMOUNT, LARGE_SPACING_AMOUNT},
	border_radius::{BORDER_RADIUS, LARGE_BORDER_RADIUS},
};

mod fonts;
pub use fonts::{strikethrough_text, BOLD_FONT};

mod theme;
pub use theme::ProjectTrackerTheme;
