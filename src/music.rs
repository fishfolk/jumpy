use bones_framework::prelude::kira::{
    sound::{
        static_sound::{StaticSoundHandle, StaticSoundSettings},
        PlaybackState, Region,
    },
    tween::Tween,
};

use crate::{prelude::*, ui::main_menu::MenuPage};

pub fn game_plugin(game: &mut Game) {
    let session = game.sessions.create(SessionNames::MUSIC_PLAYER);

    // Music player doesn't do any rendering
    session.visible = false;
    session.stages.add_system_to_stage(First, music_system);
}

/// The music playback state.
#[derive(HasSchema, Default)]
#[schema(no_clone)]
pub enum MusicState {
    /// Music is not playing.
    #[default]
    None,
    /// Playing the main menu music.
    MainMenu(StaticSoundHandle),
    /// Playing the character select music.
    CharacterSelect(StaticSoundHandle),
    /// Playing the credits music.
    Credits(StaticSoundHandle),
    /// Playing the fight music.
    Fight {
        /// The handle to the audio instance.
        instance: StaticSoundHandle,
        /// The index of the song in the shuffled playlist.
        idx: usize,
    },
}

impl MusicState {
    /// Get the current audio instance, if one is contained.
    fn current_instance(&mut self) -> Option<&mut StaticSoundHandle> {
        match self {
            MusicState::None => None,
            MusicState::MainMenu(i) => Some(i),
            MusicState::CharacterSelect(i) => Some(i),
            MusicState::Credits(i) => Some(i),
            MusicState::Fight { instance, .. } => Some(instance),
        }
    }
}

/// Bevy resource containing the in-game music playlist shuffled.
#[derive(HasSchema, Deref, DerefMut, Clone, Default)]
#[repr(C)]
pub struct ShuffledPlaylist(pub SVec<Handle<AudioSource>>);

/// The amount of time to spend fading the music in and out.
const MUSIC_FADE_DURATION: Duration = Duration::from_millis(500);

const MUSIC_VOLUME: f64 = 0.1;

/// System that plays music according to the game mode.
fn music_system(
    meta: Root<GameMeta>,
    mut audio: ResMut<AudioManager>,
    mut shuffled_fight_music: ResMutInit<ShuffledPlaylist>,
    mut music_state: ResMutInit<MusicState>,
    ctx: Res<EguiCtx>,
    sessions: Res<Sessions>,
    assets: Res<AssetServer>,
) {
    if shuffled_fight_music.is_empty() {
        let mut songs = meta.music.fight.clone();
        THREAD_RNG.with(|rng| rng.shuffle(&mut songs));
        **shuffled_fight_music = songs;
    }

    let tween = Tween {
        start_time: kira::StartTime::Immediate,
        duration: MUSIC_FADE_DURATION,
        easing: kira::tween::Easing::Linear,
    };
    let play_settings = StaticSoundSettings::default()
        .volume(MUSIC_VOLUME)
        .fade_in_tween(tween);

    // If we are in a game
    if sessions.get(SessionNames::GAME).is_some() {
        if let MusicState::Fight { instance, idx } = &mut *music_state {
            if let PlaybackState::Stopped = instance.state() {
                *idx += 1;
                *idx %= shuffled_fight_music.len();

                *instance = audio
                    .play(
                        assets
                            .get(shuffled_fight_music[*idx])
                            .0
                            .with_settings(play_settings),
                    )
                    .unwrap();
            }
        } else {
            if let Some(instance) = music_state.current_instance() {
                instance.stop(tween).unwrap();
            }

            if let Some(song) = shuffled_fight_music.get(0) {
                *music_state = MusicState::Fight {
                    instance: audio
                        .play(assets.get(*song).with_settings(play_settings))
                        .unwrap(),
                    idx: 0,
                };
            }
        }

    // If we are on a menu page
    } else if sessions.get(SessionNames::MAIN_MENU).is_some() {
        let menu_page = ctx.get_state::<MenuPage>();
        match menu_page {
            MenuPage::PlayerSelect | MenuPage::MapSelect { .. } | MenuPage::NetworkGame => {
                if !matches!(*music_state, MusicState::CharacterSelect(..)) {
                    if let Some(instance) = music_state.current_instance() {
                        instance.stop(tween).unwrap();
                    }
                    *music_state = MusicState::CharacterSelect(
                        audio
                            .play(
                                assets
                                    .get(meta.music.character_screen)
                                    .with_settings(play_settings.loop_region(Region::default())),
                            )
                            .unwrap(),
                    );
                }
            }
            MenuPage::Home | MenuPage::Settings => {
                if !matches!(*music_state, MusicState::MainMenu(..)) {
                    if let Some(instance) = music_state.current_instance() {
                        instance.stop(tween).unwrap();
                    }
                    *music_state = MusicState::MainMenu(
                        audio
                            .play(
                                assets
                                    .get(meta.music.title_screen)
                                    .with_settings(play_settings),
                            )
                            .unwrap(),
                    );
                }
            }
            MenuPage::Credits => {
                if !matches!(*music_state, MusicState::Credits(..)) {
                    if let Some(instance) = music_state.current_instance() {
                        instance.stop(tween).unwrap();
                    }
                    *music_state = MusicState::Credits(
                        audio
                            .play(assets.get(meta.music.credits).with_settings(play_settings))
                            .unwrap(),
                    );
                }
            }
        }
    }
}
