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

pub fn widget(
    mut ui: In<&mut egui::Ui>,
    meta: Root<GameMeta>,
    localization: Localization<GameMeta>,
    input: Res<GlobalPlayerControls>,
) {
    let outer_margin = egui::style::Margin::symmetric(
        ui.available_width() * 0.1,
        meta.theme.font_styles.bigger.size,
    );

    BorderedFrame::new(&meta.theme.panel.border)
        .margin(outer_margin)
        .padding(meta.theme.panel.padding)
        .show(*ui, |ui| {
            let font_color = meta.theme.panel.font_color;
            let heading_font = meta.theme.font_styles.heading.with_color(font_color);
            let heading_size = heading_font.size;
            let normal_size = meta.theme.font_styles.normal.size;
            let bigger_font = meta.theme.font_styles.bigger.with_color(font_color);
            let bigger_size = bigger_font.size;

            ui.vertical_centered(|ui| {
                ui.label(
                    meta.theme
                        .font_styles
                        .heading
                        .rich(localization.get("credits")),
                );
            });
            ui.set_min_width(ui.available_width());

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.add_space(normal_size / 2.0);

                // Back button
                if BorderedButton::themed(&meta.theme.buttons.normal, localization.get("back"))
                    .show(ui)
                    .focus_by_default(ui)
                    .clicked()
                    || input.values().any(|x| x.menu_back_just_pressed)
                {
                    ui.ctx().set_state(MenuPage::Home);
                }

                ui.with_layout(default(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        for section in &CREDITS.sections {
                            ui.add_space(heading_size / 2.0);
                            ui.label(heading_font.rich(&section.title));
                            ui.add_space(heading_size / 2.0);

                            for entry in &section.entries {
                                ui.add(egui::Separator::default().spacing(normal_size));
                                ui.horizontal(|ui| {
                                    ui.add_space(bigger_size);

                                    if let Some(name) = &entry.name {
                                        ui.label(bigger_font.rich(name));
                                    }

                                    if let Some(handle) = &entry.handle {
                                        ui.label(bigger_font.rich(&handle.name));
                                    }
                                });
                                ui.horizontal(|ui| {
                                    if let Some(contribution) = &entry.contribution {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.add_space(bigger_size);
                                                ui.label(bigger_font.rich(format!(
                                                    "{} {contribution}",
                                                    if entry.name.is_some()
                                                        || entry.handle.is_some()
                                                    {
                                                        "-"
                                                    } else {
                                                        ""
                                                    }
                                                )));
                                            },
                                        );
                                    }
                                });
                            }
                            ui.add(egui::Separator::default().spacing(normal_size));
                        }
                    });
                });
            });
        });
}
