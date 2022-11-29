use bevy_ggrs::RollbackIdProvider;
use jumpy_matchmaker_proto::TargetClient;

use crate::{
    metadata::MapMeta,
    networking::{
        client::NetClient,
        proto::{match_setup::MatchSetupMessage, ReliableGameMessageKind},
    },
    session::SessionManager,
};

use super::*;

#[derive(SystemParam)]
pub struct MapSelectMenu<'w, 's> {
    client: Option<Res<'w, NetClient>>,
    menu_page: ResMut<'w, MenuPage>,
    game: Res<'w, GameMeta>,
    commands: Commands<'w, 's>,
    localization: Res<'w, Localization>,
    map_assets: Res<'w, Assets<MapMeta>>,
    rids: ResMut<'w, RollbackIdProvider>,
    session_manager: SessionManager<'w, 's>,
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

        ui.vertical_centered_justified(|ui| {
            let bigger_text_style = &params.game.ui_theme.font_styles.bigger;
            let heading_text_style = &params.game.ui_theme.font_styles.heading;
            let small_button_style = &params.game.ui_theme.button_styles.small;

            ui.add_space(heading_text_style.size / 4.0);
            ui.themed_label(heading_text_style, &params.localization.get("local-game"));
            ui.themed_label(
                bigger_text_style,
                &params.localization.get("map-select-title"),
            );
            ui.add_space(small_button_style.font.size);

            let available_size = ui.available_size();
            let menu_width = params.game.main_menu.menu_width;
            let x_margin = (available_size.x - menu_width) / 2.0;
            let outer_margin = egui::style::Margin::symmetric(x_margin, heading_text_style.size);

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
                            for (section_title, map_handles) in [
                                (
                                    params.localization.get("default-maps"),
                                    &params.game.stable_map_handles,
                                ),
                                (
                                    params.localization.get("experimental-maps"),
                                    &params.game.experimental_map_handles,
                                ),
                            ] {
                                ui.add_space(bigger_text_style.size / 2.0);
                                ui.themed_label(bigger_text_style, &section_title);

                                // Clippy lint is a false alarm, necessary to avoid borrowing params
                                #[allow(clippy::unnecessary_to_owned)]
                                for map_handle in map_handles
                                    .iter()
                                    .map(|x| x.clone_weak())
                                    .collect::<Vec<_>>()
                                    .into_iter()
                                {
                                    let map_meta = params.map_assets.get(&map_handle).unwrap();
                                    ui.add_space(ui.spacing().item_spacing.y);

                                    if BorderedButton::themed(small_button_style, &map_meta.name)
                                        .show(ui)
                                        .clicked()
                                    {
                                        info!("Selected map, starting game");
                                        *params.menu_page = MenuPage::Home;
                                        params.commands.spawn().insert(map_handle.clone_weak());
                                        params
                                            .commands
                                            .insert_resource(NextState(GameState::InGame));
                                        params
                                            .commands
                                            .insert_resource(NextState(InGameState::Playing));
                                        params.session_manager.start_session();

                                        if let Some(client) = &mut params.client {
                                            client.send_reliable(
                                                MatchSetupMessage::SelectMap(map_handle),
                                                TargetClient::All,
                                            );
                                        }
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
        while let Some(message) = client.recv_reliable() {
            match message.kind {
                ReliableGameMessageKind::MatchSetup(setup) => match setup {
                    MatchSetupMessage::SelectMap(map_handle) => {
                        info!("Other player selected map, starting game");
                        *params.menu_page = MenuPage::Home;
                        params
                            .commands
                            .spawn()
                            .insert(map_handle)
                            .insert(Rollback::new(params.rids.next_id()));
                        params
                            .commands
                            .insert_resource(NextState(GameState::InGame));
                        params
                            .commands
                            .insert_resource(NextState(InGameState::Playing));
                        params.session_manager.start_session();
                    }
                    other => warn!("Unexpected message: {other:?}"),
                },
            }
        }
    }
}
