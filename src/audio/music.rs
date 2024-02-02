use bones_framework::prelude::kira::{
    sound::{static_sound::StaticSoundSettings, PlaybackState, Region},
    tween::Tween,
};

use crate::{prelude::*, ui::main_menu::MenuPage};

/// The music playback state.
#[derive(HasSchema, Default, PartialEq, Eq)]
#[schema(no_clone)]
pub enum MusicState {
    /// Music is not playing.
    #[default]
    None,
    /// Playing the main menu music.
    MainMenu,
    /// Playing the character select music.
    CharacterSelect,
    /// Playing the credits music.
    Credits,
    /// Playing the fight music.
    Fight {
        /// The index of the song in the shuffled playlist.
        idx: usize,
    },
}

/// Bevy resource containing the in-game music playlist shuffled.
#[derive(HasSchema, Deref, DerefMut, Clone, Default)]
#[repr(C)]
pub struct ShuffledPlaylist(pub SVec<Handle<AudioSource>>);

/// The amount of time to spend fading the music in and out.
pub const MUSIC_FADE_DURATION: Duration = Duration::from_millis(500);

pub const MUSIC_VOLUME: f64 = 0.1;

/// System that plays music according to the game mode.
pub(super) fn music_system(
    meta: Root<GameMeta>,
    mut audio: ResMut<AudioCenter>,
    mut shuffled_fight_music: ResMutInit<ShuffledPlaylist>,
    mut music_state: ResMutInit<MusicState>,
    ctx: Res<EguiCtx>,
    sessions: Res<Sessions>,
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
        if let MusicState::Fight { idx } = &mut *music_state {
            if let Some(PlaybackState::Stopped) = audio.music_state() {
                *idx = (*idx + 1) % shuffled_fight_music.len();
                audio.play_music(shuffled_fight_music[*idx], play_settings);
            }
        } else if let Some(song) = shuffled_fight_music.get(0) {
            audio.play_music(*song, play_settings);
            *music_state = MusicState::Fight { idx: 0 };
        }

    // If we are on a menu page
    } else if sessions.get(SessionNames::MAIN_MENU).is_some() {
        let menu_page = ctx.get_state::<MenuPage>();
        match menu_page {
            MenuPage::PlayerSelect | MenuPage::MapSelect { .. } | MenuPage::NetworkGame => {
                if *music_state != MusicState::CharacterSelect {
                    audio.play_music(
                        meta.music.title_screen,
                        play_settings.loop_region(Region::default()),
                    );
                    *music_state = MusicState::CharacterSelect;
                }
            }
            MenuPage::Home | MenuPage::Settings => {
                if *music_state != MusicState::MainMenu {
                    audio.play_music(meta.music.title_screen, play_settings);
                    *music_state = MusicState::MainMenu;
                }
            }
            MenuPage::Credits => {
                if *music_state != MusicState::Credits {
                    audio.play_music(meta.music.credits, play_settings);
                    *music_state = MusicState::Credits;
                }
            }
        }
    }
}
