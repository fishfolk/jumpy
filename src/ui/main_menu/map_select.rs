use crate::networking::{
    client::NetClient,
    proto::match_setup::{MatchSetupFromClient, MatchSetupFromServer},
};

use super::*;

#[derive(SystemParam)]
pub struct MapSelectMenu<'w, 's> {
    client: Option<ResMut<'w, NetClient>>,
    menu_page: ResMut<'w, MenuPage>,
    game: Res<'w, GameMeta>,
    commands: Commands<'w, 's>,
    localization: Res<'w, Localization>,
    #[system_param(ignore)]
    _phantom: PhantomData<(&'w (), &'s ())>,
}

impl<'w, 's> WidgetSystem for MapSelectMenu<'w, 's> {
    type Args = bool;

    fn system(
        world: &mut World,
        state: &mut SystemState<Self>,
        ui: &mut egui::Ui,
        _id: WidgetId,
        is_waiting: Self::Args,
    ) {
        let mut params: MapSelectMenu = state.get_mut(world);

        handle_match_setup_messages(&mut params);

        ui.vertical_centered(|ui| {
            let bigger_text_style = &params.game.ui_theme.font_styles.bigger;
            let heading_text_style = &params.game.ui_theme.font_styles.heading;
            let normal_button_style = &params.game.ui_theme.button_styles.normal;

            ui.add_space(heading_text_style.size / 4.0);
            ui.themed_label(heading_text_style, &params.localization.get("local-game"));
            ui.themed_label(
                bigger_text_style,
                &params.localization.get("map-select-title"),
            );
            ui.add_space(normal_button_style.font.size);

            let available_size = ui.available_size();
            let menu_width = params.game.main_menu.menu_width;
            let x_margin = (available_size.x - menu_width) / 2.0;
            let outer_margin = egui::style::Margin::symmetric(x_margin, 0.0);

            if is_waiting {
                ui.themed_label(
                    bigger_text_style,
                    &params.localization.get("waiting-for-map"),
                );
            } else {
                BorderedFrame::new(&params.game.ui_theme.panel.border)
                    .margin(outer_margin)
                    .padding(params.game.ui_theme.panel.padding.into())
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            // Clippy lint is a false alarm, necessary to avoid borrowing params
                            #[allow(clippy::unnecessary_to_owned)]
                            for (map_name, map_handle) in params.game.maps.to_vec().into_iter().zip(
                                params
                                    .game
                                    .map_handles
                                    .iter()
                                    .map(|x| x.clone_weak())
                                    .collect::<Vec<_>>()
                                    .into_iter(),
                            ) {
                                if ui.button(&map_name).clicked() {
                                    *params.menu_page = MenuPage::Home;
                                    params.commands.spawn().insert(map_handle.clone_weak());
                                    params
                                        .commands
                                        .insert_resource(NextState(GameState::InGame));
                                    params
                                        .commands
                                        .insert_resource(NextState(InGameState::Playing));

                                    if let Some(client) = &mut params.client {
                                        client.send_reliable(&MatchSetupFromClient::SelectMap(
                                            map_handle,
                                        ));
                                    }
                                }
                            }
                        });
                    });
            }
        });
    }
}

fn handle_match_setup_messages(params: &mut MapSelectMenu) {
    if let Some(client) = &mut params.client {
        while let Some(message) = client.recv_reliable::<MatchSetupFromServer>() {
            match message {
                MatchSetupFromServer::ClientMessage {
                    player_idx: _,
                    message: MatchSetupFromClient::SelectMap(map_handle),
                } => {
                    *params.menu_page = MenuPage::Home;
                    params.commands.spawn().insert(map_handle);
                    params
                        .commands
                        .insert_resource(NextState(GameState::InGame));
                    params
                        .commands
                        .insert_resource(NextState(InGameState::Playing));
                }
                message => {
                    warn!("Unexpected message in map select: {message:?}");
                }
            }
        }
    }
}
