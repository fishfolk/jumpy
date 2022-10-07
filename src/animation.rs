use std::time::Duration;

use bevy::reflect::FromReflect;

use crate::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AnimatedSprite>()
            .add_system_to_stage(CoreStage::PostUpdate, hydrate_animated_sprites)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                update_animated_sprite_components.after(hydrate_animated_sprites),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                animate_sprites.after(update_animated_sprite_components),
            );
    }
}

#[derive(Component, Debug, Default, Reflect, FromReflect)]
#[reflect(Component, Default)]
pub struct AnimatedSprite {
    pub start: usize,
    pub end: usize,
    pub atlas: Handle<TextureAtlas>,
    pub flip_x: bool,
    pub flip_y: bool,
    pub repeat: bool,
    pub fps: f32,
}

#[derive(Component, Default)]
struct AnimatedSpriteState {
    pub timer: Timer,
}

fn animate_sprites(
    mut animated_sprites: Query<(
        &AnimatedSprite,
        &mut TextureAtlasSprite,
        &mut AnimatedSpriteState,
    )>,
    time: Res<Time>,
) {
    for (animated_sprite, mut atlas_sprite, mut state) in &mut animated_sprites {
        state.timer.tick(time.delta());

        if state.timer.just_finished() {
            if atlas_sprite.index < animated_sprite.end {
                atlas_sprite.index += 1;
            } else {
                atlas_sprite.index = animated_sprite.start;
            }
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
            .insert(TextureAtlasSprite::default())
            .insert(AnimatedSpriteState::default());
    }
}

fn update_animated_sprite_components(
    mut animated_sprites: Query<
        (
            &AnimatedSprite,
            &mut Handle<TextureAtlas>,
            &mut TextureAtlasSprite,
            &mut AnimatedSpriteState,
        ),
        Or<(Changed<AnimatedSprite>, Added<TextureAtlasSprite>)>,
    >,
) {
    for (animated_sprite, mut atlas_handle, mut atlas_sprite, mut state) in &mut animated_sprites {
        *atlas_handle = animated_sprite.atlas.clone_weak();

        atlas_sprite.flip_x = animated_sprite.flip_x;
        atlas_sprite.flip_y = animated_sprite.flip_y;
        atlas_sprite.index = animated_sprite.start;

        state
            .timer
            .set_duration(Duration::from_secs_f32(1.0 / animated_sprite.fps));
        state.timer.set_repeating(animated_sprite.repeat);
    }
}
