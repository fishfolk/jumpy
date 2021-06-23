use macroquad::{
    color::Color,
    math::RectOffset,
    texture::Image,
    ui::{root_ui, Skin},
};

pub struct GuiResources {
    pub title_skin: Skin,
    pub login_skin: Skin,
    pub authenticating_skin: Skin,
    pub error_skin: Skin,
    pub cheat_skin: Skin,
}

impl GuiResources {
    pub fn new() -> GuiResources {
        let title_skin = {
            let label_style = root_ui()
                .style_builder()
                .font(include_bytes!("../../assets/ui/MinimalPixel v2.ttf"))
                .unwrap()
                .text_color(Color::from_rgba(255, 255, 255, 255))
                .font_size(130)
                .build();

            let button_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/button_background_2.png"),
                    None,
                ))
                .background_margin(RectOffset::new(8.0, 8.0, 12.0, 12.0))
                .margin(RectOffset::new(8.0, 8.0, 110.0, 12.0))
                .background_hovered(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/button_hovered_background_2.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/button_clicked_background_2.png"),
                    None,
                ))
                .font(include_bytes!("../../assets/ui/MinimalPixel v2.ttf"))
                .unwrap()
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(45)
                .build();

            Skin {
                label_style,
                button_style,
                ..root_ui().default_skin()
            }
        };

        let login_skin = {
            let label_style = root_ui()
                .style_builder()
                .font(include_bytes!("../../assets/ui/MinimalPixel v2.ttf"))
                .unwrap()
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(20)
                .build();

            let window_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/window_background_2.png"),
                    None,
                ))
                .background_margin(RectOffset::new(52.0, 52.0, 52.0, 52.0))
                .margin(RectOffset::new(-30.0, -30.0, -30.0, -30.0))
                .build();

            let button_style = root_ui()
                .style_builder()
                .background(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/button_background_2.png"),
                    None,
                ))
                .background_margin(RectOffset::new(8.0, 8.0, 12.0, 12.0))
                .background_hovered(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/button_hovered_background_2.png"),
                    None,
                ))
                .background_clicked(Image::from_file_with_format(
                    include_bytes!("../../assets/ui/button_clicked_background_2.png"),
                    None,
                ))
                .font(include_bytes!("../../assets/ui/MinimalPixel v2.ttf"))
                .unwrap()
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(25)
                .build();

            let tabbar_style = root_ui()
                .style_builder()
                .background_margin(RectOffset::new(8.0, 8.0, 12.0, 12.0))
                .font(include_bytes!("../../assets/ui/MinimalPixel v2.ttf"))
                .unwrap()
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
                .font(include_bytes!("../../assets/ui/MinimalPixel v2.ttf"))
                .unwrap()
                .background_margin(RectOffset::new(2., 2., 2., 2.))
                .text_color(Color::from_rgba(120, 120, 120, 255))
                .font_size(20)
                .build();

            Skin {
                label_style,
                button_style,
                tabbar_style,
                window_style,
                editbox_style,
                ..root_ui().default_skin()
            }
        };

        let authenticating_skin = {
            let label_style = root_ui()
                .style_builder()
                .font(include_bytes!("../../assets/ui/MinimalPixel v2.ttf"))
                .unwrap()
                .text_color(Color::from_rgba(200, 200, 160, 255))
                .font_size(35)
                .build();

            Skin {
                label_style,
                ..root_ui().default_skin()
            }
        };
        let error_skin = {
            let label_style = root_ui()
                .style_builder()
                .font(include_bytes!("../../assets/ui/MinimalPixel v2.ttf"))
                .unwrap()
                .text_color(Color::from_rgba(255, 0, 0, 255))
                .font_size(20)
                .build();

            Skin {
                label_style,
                ..root_ui().default_skin()
            }
        };

        let cheat_skin = root_ui().default_skin();

        GuiResources {
            title_skin,
            login_skin,
            authenticating_skin,
            error_skin,
            cheat_skin,
        }
    }
}
