use crate::prelude::*;

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("slippery_seaweed"))]
#[repr(C)]
pub struct SlipperySeaweedMeta {
    pub atlas: Handle<Atlas>,
    pub start_frame: u32,
    pub end_frame: u32,
    pub fps: f32,
    pub body_size: Vec2,
}

pub fn game_plugin(game: &mut Game) {
    SlipperySeaweedMeta::register_schema();
    game.init_shared_resource::<AssetServer>();
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(PlayerStateStage, update);
}

#[derive(Clone, Debug, HasSchema, Default)]
pub struct SlipperySeaweed;

fn hydrate(
    entities: Res<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    element_handles: Comp<ElementHandle>,
    assets: Res<AssetServer>,
    mut slippery_seaweeds: CompMut<SlipperySeaweed>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut animated_sprites: CompMut<AnimatedSprite>,
    mut bodies: CompMut<KinematicBody>,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    for entity in entities.iter_with_bitset(&not_hydrated_bitset) {
        let element_handle = element_handles.get(entity).unwrap();
        let element_meta = assets.get(element_handle.0);

        if let Ok(SlipperySeaweedMeta {
            atlas,
            body_size,
            start_frame,
            end_frame,
            fps,
        }) = assets.get(element_meta.data).try_cast_ref()
        {
            hydrated.insert(entity, MapElementHydrated);
            atlas_sprites.insert(entity, AtlasSprite::new(*atlas));
            animated_sprites.insert(
                entity,
                AnimatedSprite {
                    frames: (*start_frame..*end_frame).collect(),
                    fps: *fps,
                    repeat: true,
                    ..default()
                },
            );
            bodies.insert(
                entity,
                KinematicBody {
                    shape: ColliderShape::Rectangle { size: *body_size },
                    has_mass: false,
                    ..default()
                },
            );
            slippery_seaweeds.insert(entity, slippery_seaweed::default());
        }
    }
}

pub fn update(
    entities: Res<Entities>,
    slippery_seaweeds: CompMut<SlipperySeaweed>,
    collision_world: CollisionWorld,
    mut player_states: CompMut<PlayerState>,
) {
    for (seaweed_ent, _) in entities.iter_with(&slippery_seaweeds) {
        for (p_ent, state) in entities.iter_with(&mut player_states) {
            if collision_world
                .actor_collisions(p_ent)
                .contains(&seaweed_ent)
            {
                state.current = *incapacitated::ID;
                continue;
            }
        }
    }
}
