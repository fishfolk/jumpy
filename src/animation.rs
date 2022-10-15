use std::time::Duration;

use bevy::{ecs::system::AsSystemLabel, reflect::FromReflect, time::FixedTimestep, utils::HashMap};

use crate::prelude::*;

pub struct AnimationPlugin;

#[derive(StageLabel)]
pub enum AnimationStage {
    Hydrate,
    Animate,
}

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AnimatedSprite>()
            .register_type::<AnimationBankSprite>()
            .add_stage_after(
                CoreStage::PostUpdate,
                AnimationStage::Hydrate,
                SystemStage::single_threaded()
                    .with_system(hydrate_animation_bank_sprites)
                    .with_system(hydrate_animated_sprites),
            )
            .add_stage_after(
                AnimationStage::Hydrate,
                AnimationStage::Animate,
                SystemStage::parallel()
                    .with_run_criteria(FixedTimestep::step(crate::FIXED_TIMESTEP))
                    .with_system(update_animation_bank_sprites)
                    .with_system(
                        update_animated_sprite_components.after(update_animation_bank_sprites),
                    )
                    .with_system(
                        animate_sprites
                            .run_in_state(GameState::InGame)
                            .run_not_in_state(InGameState::Paused)
                            .after(update_animated_sprite_components.as_system_label()),
                    ),
            );
    }
}

#[derive(Component, Debug, Default, Reflect, FromReflect, Clone)]
#[reflect(Component, Default)]
pub struct AnimatedSprite {
    /// This is the current index in the animation, with an `idx` of `0` meaning that the index in
    /// the sprite sheet will be `start`.
    ///
    /// If the idx is greater than `end - start`, then the animation will loop around.
    pub index: usize,
    pub start: usize,
    pub end: usize,
    pub atlas: Handle<TextureAtlas>,
    pub flip_x: bool,
    pub flip_y: bool,
    pub repeat: bool,
    pub fps: f32,
    #[reflect(ignore)]
    pub timer: Timer,
}

/// Like an [`AnimatedSprite`] where you can chooose from multiple different animations.
#[derive(Component, Debug, Default, Reflect, FromReflect)]
#[reflect(Component, Default)]
pub struct AnimationBankSprite {
    pub current_animation: String,
    pub flip_x: bool,
    pub flip_y: bool,
    pub animations: HashMap<String, AnimatedSprite>,
    pub last_animation: String,
}

fn animate_sprites(mut animated_sprites: Query<(&mut AnimatedSprite, &mut TextureAtlasSprite)>) {
    for (mut animated_sprite, mut atlas_sprite) in &mut animated_sprites {
        animated_sprite
            .timer
            .tick(Duration::from_secs_f64(crate::FIXED_TIMESTEP));

        if animated_sprite.timer.just_finished() {
            animated_sprite.index += 1;
            animated_sprite.index %= (animated_sprite.end - animated_sprite.start).max(1);

            atlas_sprite.index = animated_sprite.start + animated_sprite.index;
        }
    }
}

fn hydrate_animation_bank_sprites(
    mut commands: Commands,
    sprites: Query<Entity, (Added<AnimationBankSprite>, Without<AnimatedSprite>)>,
) {
    for entity in &sprites {
        commands.entity(entity).insert(AnimatedSprite::default());
    }
}
fn hydrate_animated_sprites(
    mut commands: Commands,
    animated_sprites: Query<Entity, Added<AnimatedSprite>>,
) {
    for entity in &animated_sprites {
        commands
            .entity(entity)
            .insert(Handle::<TextureAtlas>::default())
            .insert(TextureAtlasSprite::default());
    }
}

fn update_animated_sprite_components(
    mut animated_sprites: Query<
        (
            &mut AnimatedSprite,
            &mut Handle<TextureAtlas>,
            &mut TextureAtlasSprite,
        ),
        Or<(Changed<AnimatedSprite>, Added<TextureAtlasSprite>)>,
    >,
) {
    for (mut animated_sprite, mut atlas_handle, mut atlas_sprite) in &mut animated_sprites {
        *atlas_handle = animated_sprite.atlas.clone_weak();

        atlas_sprite.flip_x = animated_sprite.flip_x;
        atlas_sprite.flip_y = animated_sprite.flip_y;

        let fps = animated_sprite.fps;
        let repeat = animated_sprite.repeat;
        animated_sprite
            .timer
            .set_duration(Duration::from_secs_f32(1.0 / fps.max(0.0001)));
        animated_sprite.timer.set_repeating(repeat);
    }
}

fn update_animation_bank_sprites(
    mut banks: Query<
        (&mut AnimationBankSprite, &mut AnimatedSprite),
        Or<(Changed<AnimationBankSprite>, Added<AnimatedSprite>)>,
    >,
) {
    for (mut bank, mut animated_sprite) in &mut banks {
        if bank.last_animation != bank.current_animation {
            let mut animation = bank
                .animations
                .get(&bank.current_animation)
                .cloned()
                .unwrap_or_default();

            animation.index = 0;
            if bank.flip_x {
                animation.flip_x = !animation.flip_x;
            }
            if bank.flip_y {
                animation.flip_y = !animation.flip_y;
            }

            *animated_sprite = animation;
        }
        bank.last_animation = bank.current_animation.clone();
    }
}
