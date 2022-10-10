use std::time::Duration;

use bevy::{reflect::FromReflect, time::FixedTimestep};

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
            .add_stage_after(
                CoreStage::PostUpdate,
                AnimationStage::Hydrate,
                SystemStage::single(hydrate_animated_sprites),
            )
            .add_stage_after(
                AnimationStage::Hydrate,
                AnimationStage::Animate,
                SystemStage::parallel()
                    .with_run_criteria(
                        FixedTimestep::step(crate::FIXED_TIMESTEP)
                            .chain(crate::is_in_game_run_criteria),
                    )
                    .with_system(update_animated_sprite_components)
                    .with_system(animate_sprites.after(update_animated_sprite_components)),
            );
    }
}

#[derive(Component, Debug, Default, Reflect, FromReflect)]
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

fn animate_sprites(
    mut animated_sprites: Query<(&mut AnimatedSprite, &mut TextureAtlasSprite)>,
    time: Res<Time>,
) {
    for (mut animated_sprite, mut atlas_sprite) in &mut animated_sprites {
        animated_sprite.timer.tick(time.delta());

        if animated_sprite.timer.just_finished() {
            animated_sprite.index += 1;
            animated_sprite.index %= animated_sprite.end - animated_sprite.start;

            atlas_sprite.index = animated_sprite.start + animated_sprite.index;
        }
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
            .set_duration(Duration::from_secs_f32(1.0 / fps));
        animated_sprite.timer.set_repeating(repeat);
    }
}
