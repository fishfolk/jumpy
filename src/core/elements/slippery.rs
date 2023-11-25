use crate::prelude::*;

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("slippery"))]
#[repr(C)]
pub struct SlipperyMeta {
    pub atlas: Handle<Atlas>,
    pub body_size: Vec2,
    pub player_slide: f32,
    pub body_friction: f32,
}

pub fn game_plugin(game: &mut Game) {
    SlipperyMeta::register_schema();
    game.init_shared_resource::<AssetServer>();
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PreUpdate, update);
}

#[derive(Clone, Debug, HasSchema, Default)]
pub struct Slippery {
    pub player_slide: f32,
    pub body_friction: f32,
}

fn hydrate(
    entities: Res<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    element_handles: Comp<ElementHandle>,
    assets: Res<AssetServer>,
    mut slippery: CompMut<Slippery>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    for entity in entities.iter_with_bitset(&not_hydrated_bitset) {
        let element_handle = element_handles.get(entity).unwrap();
        let element_meta = assets.get(element_handle.0);

        if let Ok(SlipperyMeta {
            atlas,
            body_size,
            player_slide,
            body_friction,
        }) = assets.get(element_meta.data).try_cast_ref()
        {
            hydrated.insert(entity, MapElementHydrated);
            atlas_sprites.insert(entity, AtlasSprite::new(*atlas));
            bodies.insert(
                entity,
                KinematicBody {
                    shape: ColliderShape::Rectangle { size: *body_size },
                    has_mass: false,
                    ..default()
                },
            );
            slippery.insert(
                entity,
                Slippery {
                    player_slide: *player_slide,
                    body_friction: *body_friction,
                },
            );
        }
    }
}

pub fn update(
    entities: Res<Entities>,
    slippery: CompMut<Slippery>,
    collision_world: CollisionWorld,
    mut bodies: CompMut<KinematicBody>,
) {
    for (slippery_ent, slippery) in entities.iter_with(&slippery) {
        for (p_ent, body) in entities.iter_with(&mut bodies) {
            if collision_world
                .actor_collisions(p_ent)
                .contains(&slippery_ent)
            {
                body.frame_friction_override = Some(slippery.body_friction);
            }
        }
    }
}
