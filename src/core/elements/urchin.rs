use crate::prelude::*;

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("urchin"))]
#[repr(C)]
pub struct UrchinMeta {
    pub image: Handle<Image>,
    pub body_diameter: f32,
    pub hit_speed: f32,
    pub gravity: f32,
    pub bounciness: f32,
    pub spin: f32,
}

pub fn game_plugin(game: &mut Game) {
    UrchinMeta::register_schema();
    game.init_shared_resource::<AssetServer>();
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update_urchins);
}

#[derive(Default, Clone, HasSchema, Debug, Copy)]
pub struct Urchin;

fn hydrate(
    mut entities: ResMutInit<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut dehrydate_bounds: CompMut<DehydrateOutOfBounds>,
    mut element_handles: CompMut<ElementHandle>,
    assets: Res<AssetServer>,
    mut urchins: CompMut<Urchin>,
    mut sprites: CompMut<Sprite>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
    mut spawner_manager: SpawnerManager,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    let spawner_entities = entities
        .iter_with_bitset(&not_hydrated_bitset)
        .collect::<Vec<_>>();

    for spawner_ent in spawner_entities {
        let transform = *transforms.get(spawner_ent).unwrap();
        let element_handle = *element_handles.get(spawner_ent).unwrap();
        let element_meta = assets.get(element_handle.0);

        if let Ok(UrchinMeta {
            image,
            body_diameter,
            bounciness,
            gravity,
            ..
        }) = assets.get(element_meta.data).try_cast_ref()
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            urchins.insert(entity, Urchin);
            sprites.insert(
                entity,
                Sprite {
                    image: *image,
                    ..Default::default()
                },
            );
            transforms.insert(entity, transform);
            element_handles.insert(entity, element_handle);
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
            spawner_manager.create_spawner(spawner_ent, vec![entity])
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
    assets: Res<AssetServer>,
) {
    for (entity, (_urchin, transform, element_handle)) in
        entities.iter_with((&mut urchins, &transforms, &element_handles))
    {
        let element_meta = assets.get(element_handle.0);
        let asset = assets.get(element_meta.data);
        let Ok(UrchinMeta {
            hit_speed, spin, ..
        }) = asset.try_cast_ref()
        else {
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
