use macroquad::{
    color::Color,
    math::RectOffset,
    texture::Image,
    ui::{root_ui, Skin},
};

pub const WINDOW_MARGIN_V: f32 = 22.0;
pub const WINDOW_MARGIN_H: f32 = 22.0;
pub const BUTTON_MARGIN_V: f32 = 8.0;
pub const BUTTON_MARGIN_H: f32 = 16.0;
pub const BUTTON_FONT_SIZE: f32 = 25.0;

pub const MENU_HEADER_FONT_SIZE: f32 = 25.0;

const WINDOW_BG_MARGIN_V: f32 = 52.0;
const WINDOW_BG_MARGIN_H: f32 = 52.0;
const BUTTON_BG_MARGIN_V: f32 = 8.0;
const BUTTON_BG_MARGIN_H: f32 = 8.0;

pub struct SkinCollection {
    pub menu: Skin,
    pub panel_group: Skin,
    pub menu_header: Skin,
    pub menu_selected: Skin,
    pub menu_disabled: Skin,
    pub map_selection: Skin,
    pub error: Skin,
    pub cheat: Skin,
}

impl SkinCollection {
    pub fn new() -> SkinCollection {
        let menu = {
            let label_style = root_ui()
                .style_builder()
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(20)
                .build();

            let window_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/window_background_2.png"),
                    None,
                ))
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
                .background(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/button_background_2.png"),
                    None,
                ))
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
                .background_hovered(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/button_hovered_background_2.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/button_clicked_background_2.png"),
                    None,
                ))
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(BUTTON_FONT_SIZE as u16)
                .build();

            let tabbar_style = root_ui()
                .style_builder()
                .background_margin(RectOffset::new(8.0, 8.0, 12.0, 12.0))
                .color(Color::from_rgba(58, 68, 102, 255))
                .color_hovered(Color::from_rgba(149, 165, 190, 255))
                .color_clicked(Color::from_rgba(129, 145, 170, 255))
                .color_selected(Color::from_rgba(139, 155, 180, 255))
                .color_selected_hovered(Color::from_rgba(149, 165, 190, 255))
                .text_color(Color::from_rgba(255, 255, 255, 255))
                .font_size(20)
                .build();

            let editbox_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/editbox_background2.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/editbox_background.png"),
                    None,
                ))
                .background_margin(RectOffset::new(2., 2., 2., 2.))
                .text_color(Color::from_rgba(120, 120, 120, 255))
                .font_size(20)
                .build();

            let checkbox_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/checkbox_background.png"),
                    None,
                ))
                .background_hovered(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/checkbox_hovered_background.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/checkbox_clicked_background.png"),
                    None,
                ))
                .build();

            let combobox_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/combobox_background.png"),
                    None,
                ))
                .background_margin(RectOffset::new(4., 25., 6., 6.))
                .text_color(Color::from_rgba(120, 120, 120, 255))
                .color(Color::from_rgba(210, 210, 210, 255))
                .font_size(25)
                .build();

            Skin {
                label_style,
                button_style,
                tabbar_style,
                window_style,
                editbox_style,
                combobox_style,
                checkbox_style,
                ..root_ui().default_skin()
            }
        };

        let menu_header = {
            let label_style = root_ui()
                .style_builder()
                .text_color(Color::from_rgba(255, 255, 255, 255))
                .font_size(MENU_HEADER_FONT_SIZE as u16)
                .build();

            Skin {
                label_style,
                ..root_ui().default_skin()
            }
        };

        let menu_selected = {
            let button_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/button_hovered_background_2.png"),
                    None,
                ))
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
                .background_hovered(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/button_hovered_background_2.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/button_clicked_background_2.png"),
                    None,
                ))
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(BUTTON_FONT_SIZE as u16)
                .build();

            Skin {
                button_style,
                ..menu.clone()
            }
        };

        let menu_disabled = {
            let button_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/button_disabled_background.png"),
                    None,
                ))
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
                .background_hovered(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/button_disabled_background.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/button_disabled_background.png"),
                    None,
                ))
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(BUTTON_FONT_SIZE as u16)
                .build();

            Skin {
                button_style,
                ..menu.clone()
            }
        };

        // Skin used in a hack to create panels.
        // Windows will not update their position if screen size changes, so in order to draw
        // windows, we will have to draw a button with a group on top.
        let panel_group = {
            let group_style = root_ui()
                .style_builder()
                .color(Color::new(0.0, 0.0, 0.0, 0.0))
                .color_hovered(Color::new(0.0, 0.0, 0.0, 0.0))
                .color_clicked(Color::new(0.0, 0.0, 0.0, 0.0))
                .build();

            let background = Image::from_file_with_format(
                include_bytes!("../../assets/ui/window_background_2.png"),
                None,
            );

            let button_style = root_ui()
                .style_builder()
                .background(background.clone())
                .background_hovered(background.clone())
                .background_clicked(background)
                .background_margin(RectOffset::new(52.0, 52.0, 52.0, 52.0))
                .build();

            Skin {
                group_style,
                button_style,
                ..root_ui().default_skin()
            }
        };

        let map_selection = {
            let label_style = root_ui()
                .style_builder()
                .text_color(Color::from_rgba(255, 255, 255, 255))
                .font_size(130)
                .build();

            let button_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/preview_background_2.png"),
                    None,
                ))
                .background_margin(RectOffset::new(52.0, 52.0, 52.0, 52.0))
                .margin(RectOffset::new(-40.0, -40.0, -40.0, -40.0))
                .background_hovered(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/preview_background_2.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/preview_background_2.png"),
                    None,
                ))
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .reverse_background_z(true)
                .font_size(45)
                .build();

            Skin {
                label_style,
                button_style,
                ..root_ui().default_skin()
            }
        };

        let error = {
            let label_style = root_ui()
                .style_builder()
                .text_color(Color::from_rgba(255, 0, 0, 255))
                .font_size(20)
                .build();

            Skin {
                label_style,
                ..root_ui().default_skin()
            }
        };

        let cheat = root_ui().default_skin();

        SkinCollection {
            menu,
            menu_header,
            menu_selected,
            menu_disabled,
            panel_group,
            map_selection,
            error,
            cheat,
        }
    }
}
