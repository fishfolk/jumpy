use macroquad::{
    prelude::*,
    ui::{root_ui, Skin},
};

use super::{ELEMENT_MARGIN, NO_COLOR};

pub struct EditorSkinCollection {
    pub default: Skin,
    pub button_disabled: Skin,
    pub window_header: Skin,
    pub menu: Skin,
    pub menu_selected: Skin,
    pub context_menu: Skin,
    pub toolbar: Skin,
    pub toolbar_bg: Skin,
    pub toolbar_header_bg: Skin,
    pub toolbar_button_disabled: Skin,
    pub tool_selector: Skin,
    pub tool_selector_selected: Skin,
    pub tileset_grid: Skin,
    pub tileset_grid_selected: Skin,
    pub tileset_subtile_grid: Skin,
    pub tileset_subtile_grid_selected: Skin,
}

impl EditorSkinCollection {
    pub const WINDOW_MARGIN_LEFT: f32 = 32.0;
    pub const WINDOW_MARGIN_RIGHT: f32 = 32.0;
    pub const WINDOW_MARGIN_TOP: f32 = 32.0;
    pub const WINDOW_MARGIN_BOTTOM: f32 = 32.0;

    pub const BUTTON_MARGIN_LEFT: f32 = 32.0;
    pub const BUTTON_MARGIN_RIGHT: f32 = 32.0;
    pub const BUTTON_MARGIN_TOP: f32 = 32.0;
    pub const BUTTON_MARGIN_BOTTOM: f32 = 32.0;

    pub fn new() -> Self {
        let default = {
            let window_background_margin = RectOffset::new(52.0, 52.0, 52.0, 52.0);
            let window_margins = RectOffset::new(
                Self::WINDOW_MARGIN_LEFT - window_background_margin.left,
                Self::WINDOW_MARGIN_RIGHT - window_background_margin.right,
                Self::WINDOW_MARGIN_TOP - window_background_margin.top,
                Self::WINDOW_MARGIN_BOTTOM - window_background_margin.bottom,
            );

            let button_background_margin = RectOffset::new(8.0, 8.0, 12.0, 12.0);
            let button_margins = RectOffset::new(
                Self::BUTTON_MARGIN_LEFT - button_background_margin.left,
                Self::BUTTON_MARGIN_RIGHT - button_background_margin.right,
                Self::BUTTON_MARGIN_TOP - button_background_margin.top,
                Self::BUTTON_MARGIN_BOTTOM - button_background_margin.bottom,
            );

            let window_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/window_background_2.png"),
                    None,
                ))
                .background_margin(window_background_margin)
                .margin(window_margins)
                .build();

            let group_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/window_background_2.png"),
                    None,
                ))
                .margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .color(NO_COLOR)
                .color_hovered(NO_COLOR)
                .color_clicked(NO_COLOR)
                .build();

            let label_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(8.0, 8.0, 4.0, 4.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(16)
                .build();

            let button_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/button_background_2.png"),
                    None,
                ))
                .background_margin(button_background_margin)
                .margin(button_margins)
                .background_hovered(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/button_hovered_background_2.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/button_clicked_background_2.png"),
                    None,
                ))
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(20)
                .build();

            let editbox_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/editbox_background2.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/editbox_background.png"),
                    None,
                ))
                .background_margin(RectOffset::new(2., 2., 2., 2.))
                .margin(RectOffset::new(0.0, 0.0, 4.0, 4.0))
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(22)
                .build();

            let checkbox_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/checkbox_background.png"),
                    None,
                ))
                .background_hovered(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/checkbox_hovered_background.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/checkbox_clicked_background.png"),
                    None,
                ))
                .build();

            let combobox_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/combobox_background.png"),
                    None,
                ))
                .background_margin(RectOffset::new(4., 25., 6., 6.))
                .text_color(Color::from_rgba(120, 120, 120, 255))
                .color(Color::from_rgba(210, 210, 210, 255))
                .font_size(22)
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
                .background(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/button_disabled_background.png"),
                    None,
                ))
                .background_margin(RectOffset::new(8.0, 8.0, 12.0, 12.0))
                .margin(RectOffset::new(8.0, 8.0, -4.0, -4.0))
                .background_hovered(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/button_disabled_background.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/button_disabled_background.png"),
                    None,
                ))
                .text_color(Color::from_rgba(88, 88, 88, 255))
                .font_size(20)
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
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(22)
                .build();

            Skin {
                label_style,
                ..default.clone()
            }
        };

        let menu = {
            let label_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(8.0, 8.0, 4.0, 4.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .text_color(Color::from_rgba(200, 200, 160, 255))
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

            Skin {
                label_style,
                button_style,
                ..default.clone()
            }
        };

        let menu_selected = {
            let label_style = root_ui()
                .style_builder()
                .margin(RectOffset::new(8.0, 8.0, 4.0, 4.0))
                .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .text_color(Color::from_rgba(200, 200, 160, 255))
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
                ..menu.clone()
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
                ..menu.clone()
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
                .text_color(Color::from_rgba(200, 200, 160, 255))
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

        let toolbar_button_disabled = {
            let button_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/button_disabled_background.png"),
                    None,
                ))
                .background_margin(RectOffset::new(8.0, 8.0, 12.0, 12.0))
                .margin(RectOffset::new(8.0, 8.0, -4.0, -4.0))
                .background_hovered(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/button_disabled_background.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/button_disabled_background.png"),
                    None,
                ))
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(20)
                .build();

            Skin {
                button_style,
                ..toolbar.clone()
            }
        };

        let tool_selector = {
            let button_style = root_ui()
                .style_builder()
                .color(Color::from_rgba(0, 0, 0, 0))
                .color_hovered(Color::from_rgba(0, 0, 0, 0))
                .color_clicked(Color::from_rgba(0, 0, 0, 0))
                .build();

            Skin {
                button_style,
                ..toolbar.clone()
            }
        };

        let tool_selector_selected = {
            let button_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/button_hovered_background_2.png"),
                    None,
                ))
                .background_margin(RectOffset::new(8.0, 8.0, 12.0, 12.0))
                .margin(RectOffset::new(8.0, 8.0, -4.0, -4.0))
                .background_hovered(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/button_hovered_background_2.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../../../assets/ui/button_hovered_background_2.png"),
                    None,
                ))
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

        EditorSkinCollection {
            default,
            button_disabled,
            window_header,
            menu,
            menu_selected,
            context_menu,
            toolbar,
            toolbar_bg,
            toolbar_header_bg,
            toolbar_button_disabled,
            tool_selector,
            tool_selector_selected,
            tileset_grid,
            tileset_grid_selected,
            tileset_subtile_grid,
            tileset_subtile_grid_selected,
        }
    }
}

impl Default for EditorSkinCollection {
    fn default() -> Self {
        Self::new()
    }
}
