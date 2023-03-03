use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update_urchins);
}

#[derive(Default, Clone, TypeUlid, Debug, Copy)]
#[ulid = "01GT7TBHRZCC1BR7T1PCR5KA4T"]
pub struct Urchin;

fn hydrate(
    mut entities: ResMut<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut dehrydate_bounds: CompMut<DehydrateOutOfBounds>,
    mut element_handles: CompMut<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut urchins: CompMut<Urchin>,
    mut sprites: CompMut<Sprite>,
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

        if let BuiltinElementKind::Urchin {
            image,
            body_diameter,
            bounciness,
            gravity,
            ..
        } = &element_meta.builtin
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            urchins.insert(entity, Urchin);
            sprites.insert(
                entity,
                Sprite {
                    image: image.clone(),
                    ..Default::default()
                },
            );
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
                    can_rotate: true,
                    shape: ColliderShape::Circle {
                        diameter: *body_diameter,
                    },
                    bounciness: *bounciness,
                    ..default()
                },
            );
        }
    }
}

fn update_urchins(
    entities: Res<Entities>,
    mut urchins: CompMut<Urchin>,
    mut bodies: CompMut<KinematicBody>,
    transforms: Comp<Transform>,
    damage_regions: Comp<DamageRegion>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
) {
    for (entity, (_urchin, transform, element_handle)) in
        entities.iter_with((&mut urchins, &transforms, &element_handles))
    {
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        let BuiltinElementKind::Urchin {
            hit_speed,
            spin,
            ..
        } = &element_meta.builtin else {
            unreachable!();
        };

        let pos = transform.translation;
        let body = bodies.get_mut(entity).unwrap();

        for (_, (damage_region, region_transform)) in
            entities.iter_with((&damage_regions, &transforms))
        {
            let region_pos = region_transform.translation;
            if damage_region
                .collider_rect(region_pos)
                .overlaps(&body.bounding_box(*transform))
            {
                body.velocity =
                    -Vec2::from_angle(Vec2::X.angle_between(region_pos.xy() - pos.xy()))
                        * *hit_speed;
                body.angular_velocity = spin * body.velocity.x.signum();
            }
        }
    }
}
