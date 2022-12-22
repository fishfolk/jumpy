use bevy::{ecs::system::AsSystemLabel, reflect::FromReflect, utils::HashMap};

use crate::prelude::*;

pub struct AnimationPlugin;

#[derive(StageLabel)]
pub enum AnimationStage {
    Hydrate,
    Animate,
}

// TODO: I don't know that I like the way this module is designed. Maybe we can simplify it.

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        // Pre-initialize components so that the scripting engine doesn't throw an error if a script
        // tries to access the component before it has been added to the world by a Rust system.
        app.world.init_component::<AnimationBankSprite>();

        app.register_type::<AnimatedSprite>()
            .register_type::<AnimationBankSprite>()
            .extend_rollback_plugin(|plugin| {
                plugin
                    .register_rollback_type::<AnimatedSprite>()
                    .register_rollback_type::<AnimationBank>()
                    .register_rollback_type::<AnimationBankSprite>()
            })
            .add_rollback_system(RollbackStage::PostUpdate, hydrate_animation_bank_sprites)
            .add_rollback_system(RollbackStage::PostUpdate, hydrate_animated_sprites)
            .add_rollback_system(RollbackStage::PostUpdate, update_animation_bank_sprites)
            .add_rollback_system(RollbackStage::PostUpdate, update_animated_sprite_components)
            .add_rollback_system(
                RollbackStage::PostUpdate,
                animate_sprites
                    .run_in_state(GameState::InGame)
                    .run_not_in_state(InGameState::Paused)
                    .after(update_animated_sprite_components.as_system_label()),
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
    pub timer: f32,
    pub stacked_atlases: Vec<Handle<TextureAtlas>>,
}

impl Clone for AnimatedSprite {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            start: self.start,
            end: self.end,
            flip_x: self.flip_x,
            flip_y: self.flip_y,
            repeat: self.repeat,
            fps: self.fps,
            atlas: self.atlas.clone_weak(),
            stacked_atlases: self.stacked_atlases.clone(),
            timer: self.timer,
        }
    }
}

/// Like an [`AnimatedSprite`] where you can chooose from multiple different animations.
#[derive(Component, Debug, Default, Reflect, FromReflect, Serialize, Deserialize, Clone)]
#[reflect(Component, Default)]
pub struct AnimationBankSprite {
    pub current_animation: String,
    pub flip_x: bool,
    pub flip_y: bool,
}

#[derive(Component, Debug, Default, Reflect, FromReflect, Clone)]
#[reflect(Component, Default)]
pub struct AnimationBank {
    pub animations: HashMap<String, AnimatedSprite>,
    pub last_animation: String,
}

#[derive(Component, Clone, Copy, Default)]
pub struct StackedAtlas;

fn animate_sprites(
    mut commands: Commands,
    mut animated_sprites: Query<(Entity, &mut AnimatedSprite), With<TextureAtlasSprite>>,
    mut texture_atlases: Query<(
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
        Option<&Parent>,
        Option<&StackedAtlas>,
    )>,
) {
    for (sprite_ent, mut animated_sprite) in &mut animated_sprites {
        let (mut atlas_sprite, ..) = texture_atlases.get_mut(sprite_ent).unwrap();

        animated_sprite.timer += 1.0 / crate::FPS as f32;
        atlas_sprite.flip_x = animated_sprite.flip_x;
        atlas_sprite.flip_y = animated_sprite.flip_y;

        if animated_sprite.timer > 1.0 / animated_sprite.fps {
            animated_sprite.timer = 0.0;
            if animated_sprite.index
                >= animated_sprite
                    .end
                    .saturating_sub(animated_sprite.start)
                    .saturating_sub(1)
                && !animated_sprite.repeat
            {
                continue;
            }
            animated_sprite.index += 1;
            animated_sprite.index %= (animated_sprite.end + 1 - animated_sprite.start).max(1);
        }

        let sprite_index = animated_sprite.start + animated_sprite.index;
        atlas_sprite.index = sprite_index;

        // Now we need to handle all the decorations
        let mut pending_stacks = animated_sprite
            .stacked_atlases
            .iter()
            .map(Some)
            .collect::<Vec<_>>();

        // If there are already spawned images for these stacked atlases, then update them
        for (mut sprite, atlas_handle, ..) in
            texture_atlases
                .iter_mut()
                .filter(|(_, _, parent, stacked)| {
                    parent.map(|x| x.get()) == Some(sprite_ent) && stacked.is_some()
                })
        {
            // Take this sprite out of the list of pending stacks
            for item in &mut pending_stacks {
                if *item == Some(atlas_handle) {
                    *item = None;
                }
            }
            sprite.flip_x = animated_sprite.flip_x;
            sprite.flip_y = animated_sprite.flip_y;
            sprite.index = sprite_index;
        }

        // For any stacked sprite that we haven't done yet
        const STACK_Z_DIFF: f32 = 0.01;
        for (i, atlas) in pending_stacks.into_iter().enumerate() {
            if let Some(atlas) = atlas {
                let stack_ent = commands
                    .spawn()
                    .insert_bundle(SpriteSheetBundle {
                        texture_atlas: atlas.clone_weak(),
                        sprite: TextureAtlasSprite {
                            index: sprite_index,
                            flip_x: animated_sprite.flip_x,
                            flip_y: animated_sprite.flip_y,
                            ..default()
                        },
                        transform: Transform::from_xyz(0.0, 0.0, (i + 1) as f32 * STACK_Z_DIFF),
                        ..default()
                    })
                    .insert(StackedAtlas)
                    .id();
                commands.entity(sprite_ent).add_child(stack_ent);
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
            .insert(TextureAtlasSprite::default());
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

fn update_animated_sprite_components(
    mut animated_sprites: Query<(
        &AnimatedSprite,
        &mut Handle<TextureAtlas>,
        &mut TextureAtlasSprite,
    )>,
) {
    for (animated_sprite, mut atlas_handle, mut atlas_sprite) in &mut animated_sprites {
        if *atlas_handle != animated_sprite.atlas {
            *atlas_handle = animated_sprite.atlas.clone_weak();
        }

        atlas_sprite.flip_x = animated_sprite.flip_x;
        atlas_sprite.flip_y = animated_sprite.flip_y;
    }
}

fn update_animation_bank_sprites(
    mut banks: Query<
        (
            &AnimationBankSprite,
            &mut AnimationBank,
            &mut AnimatedSprite,
        ),
        Or<(Changed<AnimationBankSprite>, Added<AnimatedSprite>)>,
    >,
) {
    for (bank_sprite, mut bank, mut animated_sprite) in &mut banks {
        if let Some(animation) = bank.animations.get(&bank_sprite.current_animation) {
            if bank_sprite.current_animation != bank.last_animation {
                *animated_sprite = animation.clone();
            }

            animated_sprite.flip_x = bank_sprite.flip_x && !animation.flip_x;
            animated_sprite.flip_y = bank_sprite.flip_y && !animation.flip_y;
        } else {
            warn!(
                "Trying to play non-existent animation: {}",
                bank_sprite.current_animation
            );
        }

        bank.last_animation = bank_sprite.current_animation.clone();
    }
}
