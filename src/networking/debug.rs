use async_channel::{Receiver, Sender};
use bevy::prelude::{Res, ResMut, Resource};
use bevy_egui::{
    egui::{
        self,
        plot::{Bar, BarChart, GridMark, Plot},
    },
    EguiContexts,
};
use bevy_fluent::Localization;
use ggrs::{NetworkStats, PlayerHandle};
use once_cell::sync::Lazy;

use crate::prelude::{debug_tools::ShowDebugWindows, LocalizationExt};

use super::NETWORK_MAX_PREDICTION_WINDOW;

/// Messages used by network debug channel
pub enum NetworkDebugMessage {
    /// Reset network diag on new session.
    ResetData,

    /// Notifies that frames were skipped, and the frame at start of net
    /// update loop containing skips.
    SkipFrame { frame: i32, count: u32 },
    /// Update with current frame and the last confirmed frame.
    FrameUpdate { current: i32, last_confirmed: i32 },
    /// Update that network update loop starting at this frame froze waiting
    /// for inputs from other clients after reaching max prediction window.
    FrameFroze { frame: i32 },
    /// Network stats per remote player
    NetworkStats {
        network_stats: Vec<(PlayerHandle, NetworkStats)>,
    },
}

/// Sender and receiver for [`NetworkDebugMessage`] for network diagnostics debug tool.
pub struct NetworkDebugChannel {
    pub receiver: Receiver<NetworkDebugMessage>,
    pub sender: Sender<NetworkDebugMessage>,
}

impl Default for NetworkDebugChannel {
    fn default() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Self { sender, receiver }
    }
}

pub struct NetworkFrameData {
    /// How many frames we are predicted ahead of last confirmed frame?
    pub predicted_frames: i32,
    pub current_frame: i32,
    /// Did frame freeze waiting for other inputs?
    pub froze: bool,
}

impl NetworkFrameData {
    pub fn new(confirmed: i32, current: i32) -> Self {
        Self {
            // confirmed may be -1 on start of match.
            predicted_frames: current - std::cmp::max(confirmed, 0),
            current_frame: current,
            froze: false,
        }
    }
}

/// Data captured from networking for debugging purposes.
#[derive(Resource)]
pub struct NetworkDebug {
    /// Last confirmed frame
    pub confirmed_frame: i32,
    pub current_frame: i32,

    /// Amount of frames skipped on last update that had skips.
    pub last_skipped_frame_count: u32,
    /// Last frame of net update loop that had skipped frames.
    pub last_frame_with_skips: i32,

    pub frame_buffer: Vec<NetworkFrameData>,

    /// How many frames to display in bar graph visualizer
    pub frame_buffer_display_size: usize,

    /// Is network debug tool paused.
    pub paused: bool,

    /// Network stats per connection to remote player
    pub network_stats: Vec<(PlayerHandle, NetworkStats)>,
}

impl Default for NetworkDebug {
    fn default() -> Self {
        Self {
            confirmed_frame: 0,
            current_frame: 0,
            last_skipped_frame_count: 0,
            last_frame_with_skips: -1,
            frame_buffer: vec![],
            frame_buffer_display_size: 64,
            paused: false,
            network_stats: vec![],
        }
    }
}

impl NetworkDebug {
    /// Add frame data to [`NetworkDebug`]
    pub fn add_or_update_frame(&mut self, current: i32, confirmed: i32) {
        if let Some(last) = self.frame_buffer.last_mut() {
            if last.current_frame == current {
                // Frame already buffered.
                return;
            }
        }
        self.current_frame = current;
        self.confirmed_frame = confirmed;
        self.frame_buffer
            .push(NetworkFrameData::new(confirmed, current));
    }

    /// Set frame as having frozen
    pub fn set_frozen(&mut self, frame: i32) {
        let mut last = self.frame_buffer.last_mut().unwrap();
        if last.current_frame == frame {
            last.froze = true;
        }
    }
}

/// Async channel for sending debug messages from networking implementation
/// to debug tools.
pub static NETWORK_DEBUG_CHANNEL: Lazy<NetworkDebugChannel> =
    Lazy::new(NetworkDebugChannel::default);

/// System displaying network debug window
pub fn network_debug_window(
    mut show: ResMut<ShowDebugWindows>,
    localization: Res<Localization>,
    mut diagnostics: ResMut<NetworkDebug>,
    mut egui_ctx: EguiContexts,
) {
    if show.network_debug {
        while let Ok(message) = NETWORK_DEBUG_CHANNEL.receiver.try_recv() {
            if diagnostics.paused {
                continue;
            }
            match message {
                NetworkDebugMessage::ResetData => *diagnostics = NetworkDebug::default(),
                NetworkDebugMessage::SkipFrame { frame, count } => {
                    diagnostics.last_frame_with_skips = frame;
                    diagnostics.last_skipped_frame_count = count;
                }
                NetworkDebugMessage::FrameUpdate {
                    current,
                    last_confirmed: confirmed,
                } => {
                    diagnostics.add_or_update_frame(current, confirmed);
                }
                NetworkDebugMessage::FrameFroze { frame } => {
                    diagnostics.set_frozen(frame);
                }
                NetworkDebugMessage::NetworkStats { network_stats } => {
                    diagnostics.network_stats = network_stats;
                }
            }
        }

        if show.network_debug {
            let frame_localized = localization.get("frame");
            let predicted_localized = localization.get("predicted");
            egui::Window::new(&localization.get("network-diagnostics"))
                .id(egui::Id::new("network-diagnostics"))
                .open(&mut show.network_debug)
                .show(egui_ctx.ctx_mut(), |ui| {
                    ui.monospace(&format!(
                        "{label}: {current_frame}",
                        label = localization.get("current-frame"),
                        current_frame = diagnostics.current_frame
                    ));
                    ui.monospace(&format!(
                        "{label}: {confirmed_frame}",
                        label = localization.get("confirmed-frame"),
                        confirmed_frame = diagnostics.confirmed_frame
                    ));

                    if diagnostics.last_frame_with_skips != -1 {
                        ui.monospace(&format!(
                            "{label}: {last_skip_frame}",
                            label = localization.get("last-frame-with-skips"),
                            last_skip_frame = diagnostics.last_frame_with_skips
                        ));
                        ui.monospace(&format!(
                            "{label}: {skip_count}",
                            label = localization.get("last-skipped-frame-count"),
                            skip_count = diagnostics.last_skipped_frame_count
                        ));
                    } else {
                        ui.monospace(localization.get("no-frame-skips-detected"));
                    }

                    let pause_label = if diagnostics.paused {
                        localization.get("resume")
                    } else {
                        localization.get("pause")
                    };
                    if ui.button(pause_label).clicked() {
                        diagnostics.paused = !diagnostics.paused;
                    }

                    let max_display_frame;
                    if let Some(last) = diagnostics.frame_buffer.last() {
                        max_display_frame = last.current_frame;
                    } else {
                        max_display_frame = diagnostics.frame_buffer_display_size as i32;
                    }
                    let min_display_frame =
                        max_display_frame - diagnostics.frame_buffer_display_size as i32;
                    Plot::new(localization.get("predicted-frames"))
                        .allow_zoom(false)
                        .allow_scroll(false)
                        .allow_boxed_zoom(false)
                        .include_x(min_display_frame)
                        .include_x(max_display_frame)
                        .auto_bounds_y()
                        .include_y(NETWORK_MAX_PREDICTION_WINDOW as f64)
                        .show_axes([false, true])
                        .y_grid_spacer(|_grid_input| {
                            (0..NETWORK_MAX_PREDICTION_WINDOW + 1)
                                .into_iter()
                                .map(|y| GridMark {
                                    step_size: 1.0,
                                    value: y as f64,
                                })
                                .collect()
                        })
                        .label_formatter({
                            let frame_localized = frame_localized.clone();
                            move |_name, value| {
                                let frame_floor = value.x as i32;
                                format!("{frame_localized}: {frame_floor}")
                            }
                        })
                        .height(128.0)
                        .show(ui, |plot_ui| {
                            plot_ui.bar_chart(
                                BarChart::new(
                                    diagnostics
                                        .frame_buffer
                                        .iter()
                                        .map(|frame| {
                                            let color = if frame.froze {
                                                egui::Color32::YELLOW
                                            } else {
                                                egui::Color32::LIGHT_BLUE
                                            };
                                            Bar::new(
                                                frame.current_frame as f64,
                                                frame.predicted_frames as f64,
                                            )
                                            .fill(color)
                                        })
                                        .collect(),
                                )
                                .width(1.0)
                                .element_formatter(Box::new(move |bar, _chart| {
                                    format!(
                                        "{frame_localized}: {} {predicted_localized}: {}",
                                        bar.argument as i32, bar.value as i32
                                    )
                                })),
                            );
                        });

                    for (player_handle, stats) in diagnostics.network_stats.iter() {
                        let label = format!("{} {}", localization.get("player"), player_handle);
                        ui.collapsing(label, |ui| {
                            ui.monospace(&format!("{stats:?}"));
                        });
                    }
                });
        }
    }
}
