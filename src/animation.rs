use std::time::Duration;

use bevy::reflect::FromReflect;

use crate::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AnimatedSprite>()
            .add_system_to_stage(CoreStage::PostUpdate, update_animated_sprite_components)
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

#[derive(Component)]
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

fn update_animated_sprite_components(
    mut commands: Commands,
    mut animated_sprites: Query<
        (
            Entity,
            &AnimatedSprite,
            Option<&mut Handle<TextureAtlas>>,
            Option<&mut TextureAtlasSprite>,
            Option<&mut AnimatedSpriteState>,
        ),
        Changed<AnimatedSprite>,
    >,
) {
    for (entity, animated_sprite, texture_atlas, texture_atlas_sprite, state) in
        &mut animated_sprites
    {
        let mut cmd = commands.entity(entity);
        let atlas_handle =
            // Handle::weak(HandleId::from(AssetPath::from(&animated_sprite.atlas_path)));
            animated_sprite.atlas.clone_weak();
        if let Some(mut texture) = texture_atlas {
            *texture = atlas_handle;
        } else {
            cmd.insert(atlas_handle);
        }

        if let Some(mut sprite) = texture_atlas_sprite {
            sprite.flip_x = animated_sprite.flip_x;
            sprite.flip_y = animated_sprite.flip_y;
            sprite.index = animated_sprite.start;
        } else {
            cmd.insert(TextureAtlasSprite {
                index: animated_sprite.start,
                flip_x: animated_sprite.flip_x,
                flip_y: animated_sprite.flip_y,
                ..default()
            });
        }

        if let Some(mut state) = state {
            state
                .timer
                .set_duration(Duration::from_secs_f32(1.0 / animated_sprite.fps));
            state.timer.set_repeating(animated_sprite.repeat);
        } else {
            cmd.insert(AnimatedSpriteState {
                timer: Timer::new(
                    Duration::from_secs_f32(1.0 / animated_sprite.fps),
                    animated_sprite.repeat,
                ),
            });
        }
    }
}
