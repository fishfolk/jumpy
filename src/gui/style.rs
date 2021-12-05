use macroquad::{
    color::Color,
    experimental::collections::storage,
    math::RectOffset,
    ui::{root_ui, Skin},
};

use crate::Resources;

use super::{ELEMENT_MARGIN, NO_COLOR};

pub const FONT_SIZE: f32 = 18.0;

pub const TEXT_COLOR: Color = Color {
    r: 0.78,
    g: 0.78,
    b: 0.62,
    a: 1.0,
};

pub const LABEL_MARGIN_V: f32 = 4.0;
pub const LABEL_MARGIN_H: f32 = 8.0;

pub const HEADER_FONT_SIZE: f32 = 28.0;

const BUTTON_BG_MARGIN_V: f32 = 8.0;
const BUTTON_BG_MARGIN_H: f32 = 8.0;

pub const BUTTON_MARGIN_V: f32 = 8.0;
pub const BUTTON_MARGIN_H: f32 = 16.0;

pub const BUTTON_FONT_SIZE: f32 = 25.0;

pub const SMALL_BUTTON_FONT_SIZE: f32 = 18.0;

pub const SMALL_BUTTON_MARGIN_V: f32 = 8.0;
pub const SMALL_BUTTON_MARGIN_H: f32 = 16.0;

const EDITBOX_BG_MARGIN_V: f32 = 2.0;
const EDITBOX_BG_MARGIN_H: f32 = 2.0;

pub const EDITBOX_MARGIN_V: f32 = 4.0;
pub const EDITBOX_MARGIN_H: f32 = 8.0;

const GROUP_BG_MARGIN_H: f32 = 0.0;
const GROUP_BG_MARGIN_V: f32 = 0.0;

pub const GROUP_MARGIN_H: f32 = 0.0;
pub const GROUP_MARGIN_V: f32 = 0.0;

const COMBOBOX_BG_MARGIN_H: f32 = 18.0;
const COMBOBOX_BG_MARGIN_V: f32 = 4.0;

const COMBOBOX_MARGIN_H: f32 = 8.0;
const COMBOBOX_MARGIN_V: f32 = 4.0;

const WINDOW_BG_MARGIN_V: f32 = 52.0;
const WINDOW_BG_MARGIN_H: f32 = 52.0;

pub const WINDOW_MARGIN_V: f32 = 22.0;
pub const WINDOW_MARGIN_H: f32 = 22.0;

pub const WINDOW_BG_COLOR: Color = Color {
    r: 0.15,
    g: 0.17,
    b: 0.27,
    a: 1.0,
};

pub const SELECTED_OBJECT_HIGHLIGHT_COLOR: Color = Color {
    r: 0.23,
    g: 0.67,
    b: 0.41,
    a: 1.0,
};

pub const LIST_BOX_ENTRY_HEIGHT: f32 = 24.0;

const BLANK_IMAGE_ID: &str = "blank_image";

const BUTTON_BACKGROUND_IMAGE_ID: &str = "button_background";
const BUTTON_BACKGROUND_CLICKED_IMAGE_ID: &str = "button_background_clicked";
const BUTTON_BACKGROUND_DISABLED_IMAGE_ID: &str = "button_background_disabled";
const BUTTON_BACKGROUND_HOVERED_IMAGE_ID: &str = "button_background_hovered";

const CHECKBOX_BACKGROUND_IMAGE_ID: &str = "checkbox_background";
const CHECKBOX_BACKGROUND_CHECKED_IMAGE_ID: &str = "checkbox_background_checked";
const CHECKBOX_BACKGROUND_CHECKED_HOVERED_IMAGE_ID: &str = "checkbox_background_checked_hovered";
const CHECKBOX_BACKGROUND_CLICKED_IMAGE_ID: &str = "checkbox_background_clicked";
const CHECKBOX_BACKGROUND_HOVERED_IMAGE_ID: &str = "checkbox_background_hovered";

const COMBOBOX_BACKGROUND_IMAGE_ID: &str = "combobox_background";

const EDITBOX_BACKGROUND_IMAGE_ID: &str = "editbox_background";
const EDITBOX_BACKGROUND_CLICKED_IMAGE_ID: &str = "editbox_background_clicked";

const WINDOW_BACKGROUND_IMAGE_ID: &str = "window_background";
const WINDOW_BORDER_IMAGE_ID: &str = "window_border";

pub struct SkinCollection {
    pub default: Skin,
    pub button_disabled: Skin,
    pub window_header: Skin,
    pub checkbox: Skin,
    pub checkbox_selected: Skin,
    pub label_button: Skin,
    pub list_box: Skin,
    pub list_box_selected: Skin,
    pub list_box_no_bg: Skin,
    pub context_menu: Skin,
    pub toolbar: Skin,
    pub toolbar_bg: Skin,
    pub toolbar_header_bg: Skin,
    pub toolbar_button: Skin,
    pub toolbar_button_disabled: Skin,
    pub tool_selector: Skin,
    pub tool_selector_selected: Skin,
    pub tileset_grid: Skin,
    pub tileset_grid_selected: Skin,
    pub tileset_subtile_grid: Skin,
    pub tileset_subtile_grid_selected: Skin,
    pub menu: Skin,
    pub menu_header: Skin,
    pub menu_selected: Skin,
    pub menu_disabled: Skin,
    pub map_selection: Skin,
    pub panel: Skin,
    pub panel_no_bg: Skin,
}

impl SkinCollection {
    pub fn new() -> SkinCollection {
        let resources = storage::get::<Resources>();

        let _blank_image = resources.images.get(BLANK_IMAGE_ID).unwrap();

        let button_background = resources.images.get(BUTTON_BACKGROUND_IMAGE_ID).unwrap();
        let button_background_clicked = resources
            .images
            .get(BUTTON_BACKGROUND_CLICKED_IMAGE_ID)
            .unwrap();
        let button_background_disabled = resources
            .images
            .get(BUTTON_BACKGROUND_DISABLED_IMAGE_ID)
            .unwrap();
        let button_background_hovered = resources
            .images
            .get(BUTTON_BACKGROUND_HOVERED_IMAGE_ID)
            .unwrap();

        let checkbox_background = resources.images.get(CHECKBOX_BACKGROUND_IMAGE_ID).unwrap();
        let checkbox_background_checked = resources
            .images
            .get(CHECKBOX_BACKGROUND_CHECKED_IMAGE_ID)
            .unwrap();
        let checkbox_background_checked_hovered = resources
            .images
            .get(CHECKBOX_BACKGROUND_CHECKED_HOVERED_IMAGE_ID)
            .unwrap();
        let checkbox_background_clicked = resources
            .images
            .get(CHECKBOX_BACKGROUND_CLICKED_IMAGE_ID)
            .unwrap();
        let checkbox_background_hovered = resources
            .images
            .get(CHECKBOX_BACKGROUND_HOVERED_IMAGE_ID)
            .unwrap();

        let combobox_background = resources.images.get(COMBOBOX_BACKGROUND_IMAGE_ID).unwrap();

        let editbox_background = resources.images.get(EDITBOX_BACKGROUND_IMAGE_ID).unwrap();
        let editbox_background_clicked = resources
            .images
            .get(EDITBOX_BACKGROUND_CLICKED_IMAGE_ID)
            .unwrap();

        let window_background = resources.images.get(WINDOW_BACKGROUND_IMAGE_ID).unwrap();
        let window_border = resources.images.get(WINDOW_BORDER_IMAGE_ID).unwrap();

        let default = {
            let window_style = root_ui()
                .style_builder()
                .background(window_background.image.clone())
                .background_margin(RectOffset::new(
                    WINDOW_BG_MARGIN_H,
                    WINDOW_BG_MARGIN_H,
                    WINDOW_BG_MARGIN_V,
                    WINDOW_BG_MARGIN_V,
                ))
                .margin(RectOffset::new(
                    WINDOW_MARGIN_H - WINDOW_BG_MARGIN_H,
                    WINDOW_MARGIN_H - WINDOW_BG_MARGIN_H,
                    WINDOW_MARGIN_V - WINDOW_BG_MARGIN_V,
                    WINDOW_MARGIN_V - WINDOW_BG_MARGIN_V,
                ))
                .build();

            let button_style = root_ui()
                .style_builder()
                .background(button_background.image.clone())
                .background_hovered(button_background_hovered.image.clone())
                .background_clicked(button_background_clicked.image.clone())
                .background_margin(RectOffset::new(
                    BUTTON_BG_MARGIN_H,
                    BUTTON_BG_MARGIN_H,
                    BUTTON_BG_MARGIN_V,
                    BUTTON_BG_MARGIN_V,
                ))
                .margin(RectOffset::new(
                    BUTTON_MARGIN_H - BUTTON_BG_MARGIN_H,
                    BUTTON_MARGIN_H - BUTTON_BG_MARGIN_H,
                    BUTTON_MARGIN_V - BUTTON_BG_MARGIN_V,
                    BUTTON_MARGIN_V - BUTTON_BG_MARGIN_V,
                ))
                .text_color(TEXT_COLOR)
                .font_size(BUTTON_FONT_SIZE as u16)
                .build();

            let group_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(
                    GROUP_MARGIN_H - GROUP_BG_MARGIN_H,
                    GROUP_MARGIN_H - GROUP_BG_MARGIN_H,
                    GROUP_MARGIN_V - GROUP_BG_MARGIN_V,
                    GROUP_MARGIN_V - GROUP_BG_MARGIN_V,
                ))
                .background_margin(RectOffset::new(
                    GROUP_MARGIN_H,
                    GROUP_MARGIN_H,
                    GROUP_MARGIN_V,
                    GROUP_MARGIN_V,
                ))
                .color(NO_COLOR)
                .color_hovered(NO_COLOR)
                .color_clicked(NO_COLOR)
                .build();

            let label_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(
                    LABEL_MARGIN_H,
                    LABEL_MARGIN_H,
                    LABEL_MARGIN_V,
                    LABEL_MARGIN_V,
                ))
                .text_color(TEXT_COLOR)
                .font_size(FONT_SIZE as u16)
                .build();

            let editbox_style = root_ui()
                .style_builder()
                .background(editbox_background.image.clone())
                .background_clicked(editbox_background_clicked.image.clone())
                .background_margin(RectOffset::new(
                    EDITBOX_BG_MARGIN_H,
                    EDITBOX_BG_MARGIN_H,
                    EDITBOX_BG_MARGIN_V,
                    EDITBOX_BG_MARGIN_V,
                ))
                .margin(RectOffset::new(
                    EDITBOX_MARGIN_H - EDITBOX_BG_MARGIN_H,
                    EDITBOX_MARGIN_H - EDITBOX_BG_MARGIN_H,
                    EDITBOX_MARGIN_V - EDITBOX_BG_MARGIN_V,
                    EDITBOX_MARGIN_V - EDITBOX_BG_MARGIN_V,
                ))
                .text_color(TEXT_COLOR)
                .font_size(FONT_SIZE as u16)
                .build();

            let checkbox_style = root_ui()
                .style_builder()
                .background(checkbox_background.image.clone())
                .background_hovered(checkbox_background_hovered.image.clone())
                .background_clicked(checkbox_background_clicked.image.clone())
                .build();

            let combobox_style = root_ui()
                .style_builder()
                .background(combobox_background.image.clone())
                .background_margin(RectOffset::new(
                    COMBOBOX_BG_MARGIN_H,
                    COMBOBOX_BG_MARGIN_H,
                    COMBOBOX_BG_MARGIN_V,
                    COMBOBOX_BG_MARGIN_V,
                ))
                .margin(RectOffset::new(
                    COMBOBOX_MARGIN_H - COMBOBOX_BG_MARGIN_H,
                    COMBOBOX_MARGIN_H - COMBOBOX_BG_MARGIN_H,
                    COMBOBOX_MARGIN_V - COMBOBOX_BG_MARGIN_V,
                    COMBOBOX_MARGIN_V - COMBOBOX_BG_MARGIN_V,
                ))
                .text_color(Color::from_rgba(120, 120, 120, 255))
                .color(Color::from_rgba(210, 210, 210, 255))
                .font_size(FONT_SIZE as u16)
                .build();

            let scrollbar_style = root_ui()
                .style_builder()
                .color(Color::from_rgba(38, 43, 68, 255))
                .build();

            let scrollbar_handle_style = root_ui()
                .style_builder()
                .color(Color::from_rgba(38, 43, 68, 255))
                .color_hovered(Color::from_rgba(58, 68, 102, 255))
                .color_clicked(Color::from_rgba(38, 43, 68, 255))
                .build();

            let scroll_multiplier = 10.0;
            let margin = ELEMENT_MARGIN;

            Skin {
                window_style,
                group_style,
                label_style,
                button_style,
                editbox_style,
                checkbox_style,
                combobox_style,
                scrollbar_style,
                scrollbar_handle_style,
                scroll_multiplier,
                margin,
                ..root_ui().default_skin()
            }
        };

        let button_disabled = {
            let button_style = root_ui()
                .style_builder()
                .background(button_background_disabled.image.clone())
                .background_margin(RectOffset::new(
                    BUTTON_BG_MARGIN_H,
                    BUTTON_BG_MARGIN_H,
                    BUTTON_BG_MARGIN_V,
                    BUTTON_BG_MARGIN_V,
                ))
                .margin(RectOffset::new(
                    BUTTON_MARGIN_H - BUTTON_BG_MARGIN_H,
                    BUTTON_MARGIN_H - BUTTON_BG_MARGIN_H,
                    BUTTON_MARGIN_V - BUTTON_BG_MARGIN_V,
                    BUTTON_MARGIN_V - BUTTON_BG_MARGIN_V,
                ))
                .background_hovered(button_background_disabled.image.clone())
                .background_clicked(button_background_disabled.image.clone())
                .text_color(Color::from_rgba(88, 88, 88, 255))
                .font_size(BUTTON_FONT_SIZE as u16)
                .build();

            Skin {
                button_style,
                ..default.clone()
            }
        };

        let window_header = {
            let label_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(8.0, 8.0, 4.0, 16.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .text_color(TEXT_COLOR)
                .font_size(HEADER_FONT_SIZE as u16)
                .build();

            Skin {
                label_style,
                ..default.clone()
            }
        };

        let checkbox = {
            let button_style = root_ui()
                .style_builder()
                .background(checkbox_background.image.clone())
                .background_hovered(checkbox_background_hovered.image.clone())
                .background_clicked(checkbox_background_clicked.image.clone())
                .background_margin(RectOffset::new(0.0, 0.0, 4.0, 4.0))
                .build();

            let scrollbar_style = root_ui()
                .style_builder()
                .color(NO_COLOR)
                .color_hovered(NO_COLOR)
                .color_clicked(NO_COLOR)
                .build();

            let scrollbar_handle_style = root_ui()
                .style_builder()
                .color(NO_COLOR)
                .color_hovered(NO_COLOR)
                .color_clicked(NO_COLOR)
                .build();

            let group_style = root_ui()
                .style_builder()
                .color(NO_COLOR)
                .color_hovered(NO_COLOR)
                .color_clicked(NO_COLOR)
                .build();

            Skin {
                button_style,
                scrollbar_style,
                scrollbar_handle_style,
                group_style,
                ..default.clone()
            }
        };

        let checkbox_selected = {
            let button_style = root_ui()
                .style_builder()
                .background(checkbox_background_checked.image.clone())
                .background_hovered(checkbox_background_checked_hovered.image.clone())
                .background_clicked(checkbox_background_clicked.image.clone())
                .background_margin(RectOffset::new(0.0, 0.0, 4.0, 4.0))
                .build();

            Skin {
                button_style,
                ..checkbox.clone()
            }
        };

        let label_button = {
            let button_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(0.0, 0.0, 4.0, 4.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .text_color(TEXT_COLOR)
                .font_size(FONT_SIZE as u16)
                .color(NO_COLOR)
                .color_hovered(NO_COLOR)
                .color_clicked(NO_COLOR)
                .build();

            Skin {
                button_style,
                ..default.clone()
            }
        };

        let list_box = {
            let label_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(8.0, 8.0, 4.0, 4.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .text_color(TEXT_COLOR)
                .font_size(16)
                .build();

            let button_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .color(Color::from_rgba(58, 68, 68, 255))
                .color_hovered(Color::from_rgba(58, 68, 102, 255))
                .color_clicked(Color::from_rgba(58, 68, 68, 255))
                .build();

            let scrollbar_style = root_ui()
                .style_builder()
                .color(Color::from_rgba(38, 43, 68, 255))
                .build();

            let scrollbar_handle_style = root_ui()
                .style_builder()
                .color(Color::from_rgba(38, 43, 68, 255))
                .color_hovered(Color::from_rgba(58, 68, 102, 255))
                .color_clicked(Color::from_rgba(38, 43, 68, 255))
                .build();

            Skin {
                label_style,
                button_style,
                scrollbar_style,
                scrollbar_handle_style,
                ..default.clone()
            }
        };

        let list_box_selected = {
            let label_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(8.0, 8.0, 4.0, 4.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .text_color(TEXT_COLOR)
                .font_size(16)
                .build();

            let button_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .color(Color::from_rgba(58, 68, 102, 255))
                .color_hovered(Color::from_rgba(58, 68, 102, 255))
                .color_clicked(Color::from_rgba(58, 68, 102, 255))
                .build();

            Skin {
                label_style,
                button_style,
                ..list_box.clone()
            }
        };

        let list_box_no_bg = {
            let button_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .color(Color::from_rgba(0, 0, 0, 0))
                .color_hovered(Color::from_rgba(58, 68, 102, 255))
                .color_clicked(Color::from_rgba(58, 68, 68, 255))
                .build();

            Skin {
                button_style,
                ..list_box.clone()
            }
        };

        let context_menu = {
            let button_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .color(Color::from_rgba(38, 43, 68, 255))
                .color_hovered(Color::from_rgba(38, 43, 102, 255))
                .color_clicked(Color::from_rgba(38, 43, 68, 255))
                .build();

            let scrollbar_style = root_ui()
                .style_builder()
                .color(Color::from_rgba(0, 0, 0, 0))
                .build();

            let scrollbar_handle_style = root_ui()
                .style_builder()
                .color(Color::from_rgba(0, 0, 0, 0))
                .color_hovered(Color::from_rgba(0, 0, 0, 0))
                .color_clicked(Color::from_rgba(0, 0, 0, 0))
                .build();

            Skin {
                button_style,
                scrollbar_style,
                scrollbar_handle_style,
                ..list_box.clone()
            }
        };

        let toolbar = {
            let scrollbar_style = root_ui()
                .style_builder()
                .color(Color::from_rgba(38, 43, 68, 255))
                .build();

            let scrollbar_handle_style = root_ui()
                .style_builder()
                .color(Color::from_rgba(58, 68, 68, 255))
                .color_hovered(Color::from_rgba(58, 68, 102, 255))
                .color_clicked(Color::from_rgba(58, 68, 68, 255))
                .build();

            Skin {
                scrollbar_style,
                scrollbar_handle_style,
                ..default.clone()
            }
        };

        let toolbar_bg = {
            let button_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .color(Color::from_rgba(58, 68, 68, 255))
                .color_hovered(Color::from_rgba(58, 68, 68, 255))
                .color_clicked(Color::from_rgba(58, 68, 68, 255))
                .build();

            Skin {
                button_style,
                ..toolbar.clone()
            }
        };

        let toolbar_header_bg = {
            let label_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(8.0, 8.0, 4.0, 4.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .text_color(TEXT_COLOR)
                .font_size(18)
                .build();

            let button_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .color(Color::from_rgba(38, 43, 68, 255))
                .color_hovered(Color::from_rgba(38, 43, 68, 255))
                .color_clicked(Color::from_rgba(38, 43, 68, 255))
                .build();

            Skin {
                label_style,
                button_style,
                ..toolbar.clone()
            }
        };

        let toolbar_button = {
            let button_style = root_ui()
                .style_builder()
                .background(button_background.image.clone())
                .background_hovered(button_background_hovered.image.clone())
                .background_clicked(button_background_clicked.image.clone())
                .background_margin(RectOffset::new(
                    BUTTON_BG_MARGIN_H,
                    BUTTON_BG_MARGIN_H,
                    BUTTON_BG_MARGIN_V,
                    BUTTON_BG_MARGIN_V,
                ))
                .margin(RectOffset::new(
                    SMALL_BUTTON_MARGIN_H - BUTTON_BG_MARGIN_H,
                    SMALL_BUTTON_MARGIN_H - BUTTON_BG_MARGIN_H,
                    SMALL_BUTTON_MARGIN_V - BUTTON_BG_MARGIN_V,
                    SMALL_BUTTON_MARGIN_V - BUTTON_BG_MARGIN_V,
                ))
                .text_color(TEXT_COLOR)
                .font_size(SMALL_BUTTON_FONT_SIZE as u16)
                .build();

            Skin {
                button_style,
                ..toolbar.clone()
            }
        };

        let toolbar_button_disabled = {
            let button_style = root_ui()
                .style_builder()
                .background(button_background_disabled.image.clone())
                .background_hovered(button_background_disabled.image.clone())
                .background_clicked(button_background_disabled.image.clone())
                .background_margin(RectOffset::new(
                    BUTTON_BG_MARGIN_H,
                    BUTTON_BG_MARGIN_H,
                    BUTTON_BG_MARGIN_V,
                    BUTTON_BG_MARGIN_V,
                ))
                .margin(RectOffset::new(
                    SMALL_BUTTON_MARGIN_H - BUTTON_BG_MARGIN_H,
                    SMALL_BUTTON_MARGIN_H - BUTTON_BG_MARGIN_H,
                    SMALL_BUTTON_MARGIN_V - BUTTON_BG_MARGIN_V,
                    SMALL_BUTTON_MARGIN_V - BUTTON_BG_MARGIN_V,
                ))
                .text_color(TEXT_COLOR)
                .font_size(SMALL_BUTTON_FONT_SIZE as u16)
                .build();

            Skin {
                button_style,
                ..toolbar_button.clone()
            }
        };

        let tool_selector = {
            let button_style = root_ui()
                .style_builder()
                .color(Color::from_rgba(58, 68, 68, 255))
                .color_hovered(Color::from_rgba(58, 68, 102, 255))
                .color_clicked(Color::from_rgba(58, 68, 68, 255))
                .build();

            Skin {
                button_style,
                ..toolbar.clone()
            }
        };

        let tool_selector_selected = {
            let button_style = root_ui()
                .style_builder()
                .background_margin(RectOffset::new(2.0, 2.0, 2.0, 2.0))
                .margin(RectOffset::new(6.0, 6.0, 6.0, 6.0))
                .color(Color::from_rgba(58, 68, 102, 255))
                .color_hovered(Color::from_rgba(58, 68, 102, 255))
                .color_clicked(Color::from_rgba(58, 68, 102, 255))
                .build();

            Skin {
                button_style,
                ..tool_selector.clone()
            }
        };

        let tileset_grid = {
            let button_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .color(Color::from_rgba(0, 0, 0, 0))
                .color_hovered(Color::from_rgba(38, 43, 102, 180))
                .color_clicked(Color::from_rgba(0, 0, 0, 0))
                .build();

            Skin {
                button_style,
                ..toolbar.clone()
            }
        };

        let tileset_grid_selected = {
            let button_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .color(Color::from_rgba(38, 43, 68, 180))
                .color_hovered(Color::from_rgba(38, 43, 68, 180))
                .color_clicked(Color::from_rgba(38, 43, 68, 180))
                .build();

            Skin {
                button_style,
                ..toolbar.clone()
            }
        };

        let tileset_subtile_grid = {
            let button_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .color(Color::from_rgba(0, 0, 0, 0))
                .color_hovered(Color::from_rgba(98, 43, 38, 200))
                .color_clicked(Color::from_rgba(0, 0, 0, 0))
                .build();

            Skin {
                button_style,
                ..toolbar.clone()
            }
        };

        let tileset_subtile_grid_selected = {
            let button_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .color(Color::from_rgba(98, 43, 38, 200))
                .color_hovered(Color::from_rgba(98, 43, 38, 200))
                .color_clicked(Color::from_rgba(98, 43, 38, 200))
                .build();

            Skin {
                button_style,
                ..toolbar.clone()
            }
        };

        let menu = { Skin { ..default.clone() } };

        let menu_header = {
            let label_style = root_ui()
                .style_builder()
                .text_color(TEXT_COLOR)
                .font_size(HEADER_FONT_SIZE as u16)
                .build();

            Skin {
                label_style,
                ..list_box.clone()
            }
        };

        let menu_selected = {
            let button_style = root_ui()
                .style_builder()
                .background_margin(RectOffset::new(
                    BUTTON_BG_MARGIN_H,
                    BUTTON_BG_MARGIN_H,
                    BUTTON_BG_MARGIN_V,
                    BUTTON_BG_MARGIN_V,
                ))
                .margin(RectOffset::new(
                    BUTTON_MARGIN_H - BUTTON_BG_MARGIN_H,
                    BUTTON_MARGIN_H - BUTTON_BG_MARGIN_H,
                    BUTTON_MARGIN_V - BUTTON_BG_MARGIN_V,
                    BUTTON_MARGIN_V - BUTTON_BG_MARGIN_V,
                ))
                .background(button_background_hovered.image.clone())
                .background_hovered(button_background_hovered.image.clone())
                .background_clicked(button_background_clicked.image.clone())
                .text_color(TEXT_COLOR)
                .font_size(BUTTON_FONT_SIZE as u16)
                .build();

            Skin {
                button_style,
                ..list_box.clone()
            }
        };

        let menu_disabled = {
            let button_style = root_ui()
                .style_builder()
                .background_margin(RectOffset::new(
                    BUTTON_BG_MARGIN_H,
                    BUTTON_BG_MARGIN_H,
                    BUTTON_BG_MARGIN_V,
                    BUTTON_BG_MARGIN_V,
                ))
                .margin(RectOffset::new(
                    BUTTON_MARGIN_H - BUTTON_BG_MARGIN_H,
                    BUTTON_MARGIN_H - BUTTON_BG_MARGIN_H,
                    BUTTON_MARGIN_V - BUTTON_BG_MARGIN_V,
                    BUTTON_MARGIN_V - BUTTON_BG_MARGIN_V,
                ))
                .background(button_background_disabled.image.clone())
                .background_hovered(button_background_disabled.image.clone())
                .background_clicked(button_background_disabled.image.clone())
                .text_color(TEXT_COLOR)
                .font_size(BUTTON_FONT_SIZE as u16)
                .build();

            Skin {
                button_style,
                ..list_box.clone()
            }
        };

        let panel = {
            let group_style = root_ui()
                .style_builder()
                .color(NO_COLOR)
                .color_hovered(NO_COLOR)
                .color_clicked(NO_COLOR)
                .build();

            let button_style = root_ui()
                .style_builder()
                .background(window_background.image.clone())
                .background_hovered(window_background.image.clone())
                .background_clicked(window_background.image.clone())
                .background_margin(RectOffset::new(52.0, 52.0, 52.0, 52.0))
                .build();

            Skin {
                group_style,
                button_style,
                ..default.clone()
            }
        };

        let panel_no_bg = {
            let group_style = root_ui()
                .style_builder()
                .color(NO_COLOR)
                .color_hovered(NO_COLOR)
                .color_clicked(NO_COLOR)
                .build();

            let button_style = root_ui()
                .style_builder()
                .background(window_border.image.clone())
                .background_hovered(window_border.image.clone())
                .background_clicked(window_border.image.clone())
                .background_margin(RectOffset::new(52.0, 52.0, 52.0, 52.0))
                .build();

            Skin {
                group_style,
                button_style,
                ..default.clone()
            }
        };

        let map_selection = {
            let button_style = root_ui()
                .style_builder()
                .background(window_border.image.clone())
                .background_margin(RectOffset::new(52.0, 52.0, 52.0, 52.0))
                .margin(RectOffset::new(-40.0, -40.0, -40.0, -40.0))
                .background_hovered(window_border.image.clone())
                .background_clicked(window_border.image.clone())
                .text_color(TEXT_COLOR)
                .reverse_background_z(true)
                .font_size(45)
                .build();

            Skin {
                button_style,
                ..default.clone()
            }
        };

        SkinCollection {
            default,
            button_disabled,
            window_header,
            checkbox,
            checkbox_selected,
            label_button,
            list_box,
            list_box_selected,
            list_box_no_bg,
            context_menu,
            toolbar,
            toolbar_bg,
            toolbar_header_bg,
            toolbar_button,
            toolbar_button_disabled,
            tool_selector,
            tool_selector_selected,
            tileset_grid,
            tileset_grid_selected,
            tileset_subtile_grid,
            tileset_subtile_grid_selected,
            menu,
            menu_header,
            menu_selected,
            menu_disabled,
            panel,
            panel_no_bg,
            map_selection,
        }
    }
}
