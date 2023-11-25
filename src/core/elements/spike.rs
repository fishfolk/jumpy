use crate::prelude::*;

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("spike"))]
#[repr(C)]
pub struct SpikeMeta {
    pub atlas: Handle<Atlas>,
    pub body_size: Vec2,
    pub start_frame: u32,
    pub end_frame: u32,
    pub fps: f32,
}

pub fn game_plugin(game: &mut Game) {
    SpikeMeta::register_schema();
    game.init_shared_resource::<AssetServer>();
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update);
}

#[derive(Clone, Debug, HasSchema, Default)]
pub struct Spike;

fn hydrate(
    entities: Res<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    element_handles: Comp<ElementHandle>,
    assets: Res<AssetServer>,
    mut spikes: CompMut<Spike>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut animated_sprites: CompMut<AnimatedSprite>,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    for entity in entities.iter_with_bitset(&not_hydrated_bitset) {
        let element_handle = element_handles.get(entity).unwrap();
        let element_meta = assets.get(element_handle.0);

        if let Ok(SpikeMeta {
            atlas,
            body_size,
            start_frame,
            end_frame,
            fps,
            ..
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
            spikes.insert(entity, spike::default());
        }
    }
}

fn update(
    entities: Res<Entities>,
    mut spikes: CompMut<Spike>,
    collision_world: CollisionWorld,
    player_indexes: Comp<PlayerIdx>,
    invincibles: CompMut<Invincibility>,
    mut commands: Commands,
    transforms: Comp<Transform>,
) {
    for (entity, (_, pos)) in entities.iter_with((&mut spikes, &transforms)) {
        collision_world
            .actor_collisions_filtered(entity, |e| {
                player_indexes.contains(e) && invincibles.get(e).is_none()
            })
            .into_iter()
            .for_each(|player| {
                commands.add(PlayerCommand::kill(player, Some(pos.translation.xy())));
            });
    }
}
