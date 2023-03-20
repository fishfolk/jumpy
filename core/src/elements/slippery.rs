use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PreUpdate, update);
}

#[derive(Clone, Debug, TypeUlid, Default)]
#[ulid = "01GTF77BGTAPJNTYEXKP6A2862"]
pub struct Slippery {
    pub player_slide: f32,
    pub body_friction: f32,
}

fn hydrate(
    entities: Res<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut slippery: CompMut<Slippery>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    for entity in entities.iter_with_bitset(&not_hydrated_bitset) {
        let element_handle = element_handles.get(entity).unwrap();
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        if let BuiltinElementKind::Slippery {
            atlas,
            body_size,
            player_slide,
            body_friction,
        } = &element_meta.builtin
        {
            hydrated.insert(entity, MapElementHydrated);
            atlas_sprites.insert(entity, AtlasSprite::new(atlas.clone()));
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
