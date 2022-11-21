use bevy::ecs::{schedule::ShouldRun, system::SystemParam};
use bevy_ggrs::{ggrs::SyncTestSession, ResetGGRSSession, SessionType};

use crate::{
    loading::PlayerInputCollector, map::elements::player_spawner::CurrentPlayerSpawner, prelude::*,
    run_criteria::ShouldRunExt, ui::input::MenuAction, GgrsConfig,
};

pub mod event;

pub struct UtilsPlugin;

impl Plugin for UtilsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Sort>()
            .extend_rollback_plugin(|plugin| plugin.register_rollback_type::<Sort>());
    }
}

/// Cache a string using [`wasm_bindgen::intern`] when running on web platforms.
///
/// [`wasm_bindgen::intern`]: https://docs.rs/wasm-bindgen/latest/wasm_bindgen/fn.intern.html
#[allow(unused)]
#[inline]
pub fn cache_str(s: &str) {
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen::intern(s);
}

/// A [`Component`] that is simply an index that may be used to sort elements for deterministic
/// iteration.
#[derive(
    Deref, DerefMut, Component, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Reflect, Default,
)]
#[reflect(Component)]
pub struct Sort(pub u32);

/// Returns the hypothetical "invalid entity" ( `Entity::from_raw(u32::MAX)` ).
///
/// This serves as a workaround for the fact that [`Entity`] does not implement [`Default`], but
/// [`Default`] is required to reflect [`Component`].
///
/// It would be best to find a way to get rid of this, but I ( @zicklag ) belive that the chances of
/// the invalid entity turning out to be a real entity that happened to get all the way up to
/// index [`u32::MAX`] is highly unlikely.
#[inline]
pub fn invalid_entity() -> Entity {
    Entity::from_raw(u32::MAX)
}

/// System parameter that can be used to reset the game world.
///
/// Currently this just means de-spawning all of the entities other than the camera and resetting
/// the camera position, but in the future this process might be more involved so we centralize the
/// logic here so it can be re-used.
#[derive(SystemParam)]
pub struct ResetManager<'w, 's> {
    commands: Commands<'w, 's>,
    camera: Query<
        'w,
        's,
        (
            &'static mut Camera,
            &'static mut Transform,
            &'static mut OrthographicProjection,
        ),
        With<Camera>,
    >,
    entities_to_despawn: Query<
        'w,
        's,
        Entity,
        (
            Without<Camera>,
            Without<PlayerInputCollector>,
            Without<ActionState<MenuAction>>,
        ),
    >,
    current_player_spawner: ResMut<'w, CurrentPlayerSpawner>,
}

impl<'w, 's> ResetManager<'w, 's> {
    /// Clean up the game world, despawning all the gameplay entities, but leaving necessary
    /// entities like camera.
    pub fn reset_world(&mut self) {
        // Clean up all entities other than the camera and the player entities
        for entity in self.entities_to_despawn.iter() {
            self.commands.entity(entity).despawn_recursive();
        }

        // Reset camera position
        if let Some((mut camera, mut transform, mut projection)) = self.camera.iter_mut().next() {
            camera.viewport = default();
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
            projection.scale = 1.0;
        }

        **self.current_player_spawner = 0;

        // Clear the game session
        self.commands.insert_resource(ResetGGRSSession);
        self.commands.remove_resource::<SessionType>();
        self.commands
            .remove_resource::<SyncTestSession<GgrsConfig>>();
    }
}

/// Heper stage run criteria that only runs if we are in a gameplay state.
pub fn is_in_game_run_criteria(
    game_state: Option<Res<CurrentState<GameState>>>,
    in_game_state: Option<Res<CurrentState<InGameState>>>,
) -> ShouldRun {
    let is_in_game = game_state
        .map(|x| x.0 == GameState::InGame)
        .unwrap_or(false)
        && in_game_state
            .map(|x| x.0 != InGameState::Paused)
            .unwrap_or(false);

    ShouldRun::new(is_in_game, false)
}
