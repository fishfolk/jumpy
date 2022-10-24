use crate::player::MAX_PLAYERS;

use super::*;

#[derive(SystemParam)]
pub struct MatchmakingMenu<'w, 's> {
    commands: Commands<'w, 's>,
    menu_page: ResMut<'w, MenuPage>,
    game: Res<'w, GameMeta>,
    localization: Res<'w, Localization>,
    state: Local<'s, State>,
}

pub struct State {
    player_count: usize,
    status: Status,
}

enum Status {
    NotConnected,
    Connecting { players: usize },
    Finished,
}

impl Default for State {
    fn default() -> Self {
        Self {
            player_count: 4,
            status: Status::NotConnected,
        }
    }
}

impl<'w, 's> WidgetSystem for MatchmakingMenu<'w, 's> {
    type Args = ();
    fn system(
        world: &mut World,
        state: &mut SystemState<Self>,
        ui: &mut egui::Ui,
        _: WidgetId,
        _: (),
    ) {
        let mut params: MatchmakingMenu = state.get_mut(world);

        let bigger_text_style = &params.game.ui_theme.font_styles.bigger;
        let heading_text_style = &params.game.ui_theme.font_styles.heading;
        let normal_button_style = &params.game.ui_theme.button_styles.normal;

        ui.vertical_centered(|ui| {
            ui.add_space(heading_text_style.size / 4.0);
            ui.themed_label(heading_text_style, &params.localization.get("online-game"));
            ui.themed_label(
                bigger_text_style,
                &params.localization.get("configure-match"),
            );
            ui.add_space(heading_text_style.size * 4.0);
        });

        let available_size = ui.available_size();
        let menu_width = params.game.main_menu.menu_width;
        let x_margin = (available_size.x - menu_width) / 2.0;
        let outer_margin = egui::style::Margin::symmetric(x_margin, 0.0);

        BorderedFrame::new(&params.game.ui_theme.panel.border)
            .margin(outer_margin)
            .padding(params.game.ui_theme.panel.padding.into())
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.themed_label(
                        bigger_text_style,
                        &format!("{}: ", params.localization.get("player-count")),
                    );
                    ui.scope(|ui| {
                        ui.set_enabled(params.state.player_count > 1);
                        if BorderedButton::themed(normal_button_style, "-")
                            .focus_on_hover(false)
                            .show(ui)
                            .clicked()
                        {
                            params.state.player_count -= 1;
                        }
                    });
                    ui.themed_label(bigger_text_style, &params.state.player_count.to_string());
                    ui.scope(|ui| {
                        ui.set_enabled(params.state.player_count < MAX_PLAYERS);
                        if BorderedButton::themed(normal_button_style, "+")
                            .focus_on_hover(false)
                            .show(ui)
                            .clicked()
                        {
                            params.state.player_count += 1;
                        }
                    });
                });

                ui.add_space(normal_button_style.font.size);
                ui.horizontal(|ui| {
                    if BorderedButton::themed(
                        normal_button_style,
                        &params.localization.get("cancel"),
                    )
                    .focus_on_hover(false)
                    .show(ui)
                    .clicked()
                    {
                        *params.menu_page = MenuPage::Home;
                    }

                    if BorderedButton::themed(
                        normal_button_style,
                        &params.localization.get("search-for-match"),
                    )
                    .focus_on_hover(false)
                    .show(ui)
                    .clicked()
                    {}
                });
            });
    }
}
