use super::*;

const CREDITS_STR: &str = include_str!("../../../CREDITS.md");

static CREDITS: Lazy<Credits> = Lazy::new(|| credits_parser::credits(CREDITS_STR).unwrap());

pub struct Credits {
    pub sections: Vec<CreditsSection>,
}

pub struct CreditsSection {
    pub title: String,
    pub entries: Vec<CreditsEntry>,
}

pub struct CreditsEntry {
    pub name: Option<String>,
    pub handle: Option<CreditsHandle>,
    pub contribution: Option<String>,
}

pub struct CreditsHandle {
    pub name: String,
    pub link: String,
}

peg::parser! {
    grammar credits_parser() for str {
      rule _ = " "*
      rule __ = [' ' | '\n']*

      rule handle() -> CreditsHandle
          = "[" name:$([^']']*) "](" link:$([^')']*) ")" {
              CreditsHandle { name: name.trim().into(), link: link.trim().into() }
          }

      rule entry_name() -> String
          = name:$([^'[' | '-' | '\n']*) { name.trim().into() }

      rule contribution() -> String
          = "-" _ contribution:$([^'\n']*) {
            contribution.trim().into()
          }

      rule entry() -> CreditsEntry
          = "-" _ name:entry_name()? _ handle:handle()? _ contribution:contribution()? "\n" {
            CreditsEntry { name, handle, contribution }
          }

      rule section() -> CreditsSection
          = "##" _ title:$([^'\n']*) __
            entries:(entry() ** __)
          {
            CreditsSection { title: title.trim().into(), entries }
          }

      pub rule credits() -> Credits
          = "# Credits" __ sections:(section() ** __) __ {
            Credits {
                sections,
            }
          }
    }
}

#[derive(SystemParam)]
pub struct CreditsMenu<'w, 's> {
    game: Res<'w, GameMeta>,
    menu_page: ResMut<'w, MenuPage>,
    localization: Res<'w, Localization>,
    keyboard_input: Res<'w, Input<KeyCode>>,
    menu_input: Query<'w, 's, &'static mut ActionState<MenuAction>>,
}

impl<'w, 's> WidgetSystem for CreditsMenu<'w, 's> {
    type Args = ();

    fn system(
        world: &mut World,
        state: &mut SystemState<Self>,
        ui: &mut egui::Ui,
        _id: WidgetId,
        _args: Self::Args,
    ) {
        let mut params: CreditsMenu = state.get_mut(world);

        let ui_theme = &params.game.ui_theme;
        let heading_font = ui_theme
            .font_styles
            .heading
            .colored(ui_theme.panel.font_color);
        let bigger_font = ui_theme
            .font_styles
            .bigger
            .colored(ui_theme.panel.font_color);
        let normal_font = ui_theme
            .font_styles
            .normal
            .colored(ui_theme.panel.font_color);

        let outer_margin =
            egui::style::Margin::symmetric(ui.available_width() * 0.1, bigger_font.size);

        BorderedFrame::new(&ui_theme.panel.border)
            .margin(outer_margin)
            .padding(ui_theme.panel.padding.into())
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.themed_label(&heading_font, &params.localization.get("credits"));
                });
                ui.set_min_width(ui.available_width());

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    // Back button
                    let back_button = BorderedButton::themed(
                        &params.game.ui_theme.button_styles.normal,
                        &params.localization.get("back"),
                    )
                    .show(ui)
                    .focus_by_default(ui);

                    ui.add_space(normal_font.size / 2.0);

                    if back_button.clicked()
                        || params.menu_input.single().just_pressed(MenuAction::Back)
                        || params.keyboard_input.just_pressed(KeyCode::Escape)
                    {
                        *params.menu_page = MenuPage::Home;
                    }

                    ui.with_layout(default(), |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            for section in &CREDITS.sections {
                                ui.add_space(heading_font.size / 2.0);
                                ui.themed_label(&heading_font, &section.title);
                                ui.add_space(heading_font.size / 2.0);

                                for entry in &section.entries {
                                    ui.add(egui::Separator::default().spacing(normal_font.size));
                                    ui.horizontal(|ui| {
                                        ui.add_space(bigger_font.size);

                                        if let Some(name) = &entry.name {
                                            ui.themed_label(&bigger_font, name);
                                        }

                                        if let Some(handle) = &entry.handle {
                                            ui.themed_label(&bigger_font, &handle.name);
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        if let Some(contribution) = &entry.contribution {
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.add_space(bigger_font.size);
                                                    ui.themed_label(
                                                        &bigger_font,
                                                        &format!(
                                                            "{} {contribution}",
                                                            if entry.name.is_some()
                                                                || entry.handle.is_some()
                                                            {
                                                                "-"
                                                            } else {
                                                                ""
                                                            }
                                                        ),
                                                    );
                                                },
                                            );
                                        }
                                    });
                                }
                                ui.add(egui::Separator::default().spacing(normal_font.size));
                            }
                        });
                    });
                });
            });
    }
}
