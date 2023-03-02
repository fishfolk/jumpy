use std::collections::HashMap;

use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update_snails);
}

#[derive(Clone, TypeUlid, Debug, Copy)]
#[ulid = "01GTA646V5SESHBAP325MNT22Y"]
pub enum Snail {
    Hiding(f32),
    UnHiding,
    Moving {
        attempted_x: Option<f32>,
        frame_timer: f32,
        anim_index: usize,
    },
}

fn hydrate(
    mut entities: ResMut<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut dehrydate_bounds: CompMut<DehydrateOutOfBounds>,
    mut element_handles: CompMut<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut snails: CompMut<Snail>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut animated_sprites: CompMut<AnimatedSprite>,
    mut animation_banks: CompMut<AnimationBankSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    let spawners = entities
        .iter_with_bitset(&not_hydrated_bitset)
        .collect::<Vec<_>>();

    for spawner_ent in spawners {
        let transform = *transforms.get(spawner_ent).unwrap();
        let element_handle = element_handles.get(spawner_ent).unwrap();
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        if let BuiltinElementKind::Snail {
            atlas,
            fps,
            body_diameter,
            gravity,
            bounciness,
            hide_time,
            hide_frames,
            ..
        } = &element_meta.builtin
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            snails.insert(entity, Snail::Hiding(*hide_time));
            transforms.insert(entity, transform);
            element_handles.insert(entity, element_handle.clone());
            hydrated.insert(entity, MapElementHydrated);
            dehrydate_bounds.insert(entity, DehydrateOutOfBounds(spawner_ent));

            bodies.insert(
                entity,
                KinematicBody {
                    gravity: *gravity,
                    has_mass: true,
                    has_friction: true,
                    bounciness: *bounciness,
                    shape: ColliderShape::Circle {
                        diameter: *body_diameter,
                    },
                    ..default()
                },
            );
            atlas_sprites.insert(entity, AtlasSprite::new(atlas.clone()));
            animated_sprites.insert(entity, default());

            let mut animations = HashMap::new();

            animations.insert(key!("disabled"), default());
            animations.insert(
                key!("hide"),
                AnimatedSprite {
                    frames: hide_frames.iter().cloned().collect(),
                    fps: *fps,
                    repeat: false,
                    ..default()
                },
            );
            animations.insert(
                key!("unhide"),
                AnimatedSprite {
                    frames: hide_frames.iter().rev().cloned().collect(),
                    fps: *fps,
                    repeat: false,
                    ..default()
                },
            );
            animation_banks.insert(
                entity,
                AnimationBankSprite {
                    current: key!("hide"),
                    animations: Arc::new(animations),
                    last_animation: key!("hide"),
                },
            );
        }
    }
}

fn update_snails(
    entities: Res<Entities>,
    mut snails: CompMut<Snail>,
    mut sprites: CompMut<AtlasSprite>,
    mut animated_sprites: CompMut<AnimatedSprite>,
    mut animation_banks: CompMut<AnimationBankSprite>,
    frame_time: Res<FrameTime>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    damage_regions: CompMut<DamageRegion>,
) {
    for (entity, (snail, element_handle, body, sprite)) in
        entities.iter_with((&mut snails, &element_handles, &mut bodies, &mut sprites))
    {
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        let BuiltinElementKind::Snail {
            fps,
            bounciness,
            hide_time,
            hit_speed,
            crawl_frames,
            move_frame_indexes,
            hide_frames,
            ..
        } = &element_meta.builtin else {
            unreachable!();
        };
        let Some(animated_sprite) = animated_sprites.get_mut(entity) else { continue };
        let Some(animation_bank) = animation_banks.get_mut(entity) else { continue };

        let mut hit = false;

        for (_, (damage_region, region_transform)) in
            entities.iter_with((&damage_regions, &transforms))
        {
            let transform = transforms.get(entity).unwrap();
            let region_pos = region_transform.translation;
            if damage_region
                .collider_rect(region_pos)
                .overlaps(&body.bounding_box(*transform))
            {
                hit = true;
                body.velocity = -Vec2::from_angle(
                    Vec2::X.angle_between(region_pos.xy() - transform.translation.xy()),
                ) * *hit_speed;
            }
        }

        match &snail {
            Snail::UnHiding | Snail::Moving { .. } => {
                if hit {
                    *snail = Snail::Hiding(0.0);
                    body.bounciness = *bounciness;
                    animation_bank.current = key!("hide");
                }
            }
            _ => {}
        }

        match snail {
            Snail::Hiding(time) => {
                if hit {
                    *time = 0.0;
                    sprite.flip_x = !sprite.flip_x;
                } else {
                    *time += 1.0 / crate::FPS;
                    if *time >= *hide_time {
                        *snail = Snail::UnHiding;
                        animation_bank.current = key!("unhide");
                    }
                }
            }
            Snail::UnHiding => {
                if animated_sprite.index == hide_frames.len() - 1 {
                    *snail = Snail::Moving {
                        attempted_x: None,
                        frame_timer: 0.0,
                        anim_index: 0,
                    };
                    sprite.index = 0;
                    animation_bank.current = key!("disabled");
                    body.bounciness = 0.0;
                }
            }
            Snail::Moving {
                attempted_x,
                frame_timer,
                anim_index,
            } => {
                let translation = &mut transforms.get_mut(entity).unwrap().translation;

                if let Some(attempted_x) = attempted_x {
                    if attempted_x != &translation.x {
                        *snail = Snail::Hiding(*hide_time);
                        body.bounciness = *bounciness;
                        animation_bank.current = key!("hide");
                        sprite.flip_x = !sprite.flip_x;
                        continue;
                    }
                }

                *frame_timer += **frame_time;

                if *frame_timer > 1.0 / fps.max(f32::MIN_POSITIVE) {
                    *frame_timer = 0.0;
                    *anim_index = (*anim_index + 1) % crawl_frames.len();
                    sprite.index = crawl_frames[*anim_index];

                    if move_frame_indexes.iter().any(|i| i == anim_index) {
                        let next_x = translation.x - 1.0 * if sprite.flip_x { -1.0 } else { 1.0 };
                        *attempted_x = Some(next_x);
                        translation.x = next_x;
                    }
                } else {
                    *attempted_x = None
                }
            }
        }
    }
}
