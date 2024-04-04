use crate::{prelude::*, PackMeta};

use super::main_menu::MenuPage;

#[derive(Clone, Debug, Default)]
pub enum MapSelectAction {
    #[default]
    None,
    SelectMap(MapPool),
    GoBack,
}

/// Network message that may be sent when selecting a map.
#[derive(Serialize, Deserialize)]
pub enum MapSelectMessage {
    SelectMap(NetworkHandle<MapMeta>),
}

pub fn map_select_menu(
    asset_server: Res<AssetServer>,
    meta: Root<GameMeta>,
    ctx: Res<EguiCtx>,
    localization: Localization<GameMeta>,
    player_controls: Res<GlobalPlayerControls>,
    rng: ResInit<GlobalRng>,
) -> MapSelectAction {
    if player_controls.values().any(|x| x.menu_back_just_pressed) {
        return MapSelectAction::GoBack;
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(&ctx, |ui| {
            let screen_rect = ui.max_rect();

            let pause_menu_width = meta.main_menu.menu_width;
            let x_margin = (screen_rect.width() - pause_menu_width) / 2.0;
            let outer_margin = egui::style::Margin::symmetric(x_margin, screen_rect.height() * 0.1);

            BorderedFrame::new(&meta.theme.panel.border)
                .margin(outer_margin)
                .padding(meta.theme.panel.padding)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());

                    ui.vertical_centered(|ui| {
                        ui.label(
                            meta.theme
                                .font_styles
                                .bigger
                                .rich(localization.get("map-select-title")),
                        );
                    });

                    ui.add_space(meta.theme.font_styles.normal.size);

                    let menu_page_state = ui.ctx().get_state::<MenuPage>();
                    let is_waiting = match menu_page_state {
                        MenuPage::MapSelect { is_waiting } => is_waiting,
                        // If we are not on the map select menu page it means we are selecting
                        // a map from the pause menu.
                        _ => false,
                    };
                    if is_waiting {
                        ui.label(
                            meta.theme
                                .font_styles
                                .bigger
                                .rich(localization.get("waiting-for-map")),
                        );

                        MapSelectAction::None
                    } else {
                        egui::ScrollArea::vertical()
                            .show(ui, |ui| {
                                ui.vertical_centered_justified(|ui| {
                                    let all_maps_button = BorderedButton::themed(
                                        &meta.theme.buttons.small,
                                        "All Maps".to_string(),
                                    )
                                    .show(ui)
                                    .focus_by_default(ui);

                                    if all_maps_button.clicked() {
                                        // TODO: Currently does not include packs, only stable maps.
                                        let mut map_pool =
                                            MapPool::from_slice(&meta.core.stable_maps);
                                        map_pool.randomize_current_map(&rng);
                                        return MapSelectAction::SelectMap(map_pool);
                                    }

                                    for handle in meta.core.stable_maps.iter() {
                                        let map_meta = asset_server.get(*handle);

                                        let button = BorderedButton::themed(
                                            &meta.theme.buttons.small,
                                            map_meta.name.to_string(),
                                        )
                                        .show(ui);

                                        if button.clicked() {
                                            return MapSelectAction::SelectMap(
                                                MapPool::from_single_map(*handle),
                                            );
                                        }
                                    }

                                    for pack in asset_server.packs() {
                                        let pack_meta =
                                            asset_server.get(pack.root.typed::<PackMeta>());
                                        for map in pack_meta.maps.iter() {
                                            let map_meta = asset_server.get(*map);
                                            let button = BorderedButton::themed(
                                                &meta.theme.buttons.small,
                                                map_meta.name.to_string(),
                                            )
                                            .show(ui);

                                            if button.clicked() {
                                                return MapSelectAction::SelectMap(
                                                    MapPool::from_single_map(*map),
                                                );
                                            }
                                        }
                                    }

                                    MapSelectAction::None
                                })
                                .inner
                            })
                            .inner
                    }
                })
                .inner
        })
        .inner
}
