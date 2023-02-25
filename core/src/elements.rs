use crate::prelude::*;

pub mod crab;
pub mod crate_item;
pub mod decoration;
pub mod grenade;
pub mod kick_bomb;
pub mod mine;
pub mod musket;
pub mod player_spawner;
pub mod sproinger;
pub mod stomp_boots;
pub mod sword;

/// Marker component added to map elements that have been hydrated.
#[derive(Clone, TypeUlid)]
#[ulid = "01GP42Q5GCY5Y4JC7SQ1YRHYKN"]
pub struct MapElementHydrated;

/// Component that contains the [`Entity`] to de-hydrate when the entity with this component is out
/// of the [`LoadedMap`] bounds.
///
/// This is useful for map elements that spawn items: when the item falls off the map, it should
/// de-hydrate it's spawner, so that the spawner will re-spawn the item in it's default state.
#[derive(Clone, TypeUlid, Deref, DerefMut)]
#[ulid = "01GP9NY0Y50Y2A8M4A7E9NN8VE"]
pub struct DehydrateOutOfBounds(pub Entity);

/// Component containing an element's metadata handle.
#[derive(Clone, TypeUlid, Deref, DerefMut, Default)]
#[ulid = "01GP421CHN323T2614F19PA5E9"]
pub struct ElementHandle(pub Handle<ElementMeta>);

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::Last, throw_dropped_items);
    session
        .stages
        .add_system_to_stage(CoreStage::First, handle_out_of_bounds_items);

    decoration::install(session);
    player_spawner::install(session);
    sproinger::install(session);
    sword::install(session);
    grenade::install(session);
    crab::install(session);
    kick_bomb::install(session);
    mine::install(session);
    musket::install(session);
    stomp_boots::install(session);
    crate_item::install(session);
}

fn handle_out_of_bounds_items(
    mut commands: Commands,
    mut hydrated: CompMut<MapElementHydrated>,
    entities: ResMut<Entities>,
    transforms: CompMut<Transform>,
    spawners: Comp<DehydrateOutOfBounds>,
    map: Res<LoadedMap>,
) {
    for (item_ent, (transform, spawner)) in entities.iter_with((&transforms, &spawners)) {
        if map.is_out_of_bounds(&transform.translation) {
            hydrated.remove(**spawner);
            commands.add(move |mut entities: ResMut<Entities>| {
                entities.kill(item_ent);
            });
        }
    }
}

/// Component defining the strength of the throw types when an item is dropped.
///
/// Mainly handled by the [`throw_dropped_items`] system.
#[derive(Clone, Copy, TypeUlid)]
#[ulid = "01GSGE6N4TSEMQ1DKDP5Y66TE4"]
pub struct ItemThrow {
    normal: Vec2,
    fast: Vec2,
    up: Vec2,
    drop: Vec2,
    lob: Vec2,
    roll: Vec2,
    spin: f32,
}

impl ItemThrow {
    /// Generally what to start with for throw velocities. The standard `spin` is `0.0`.
    fn standard() -> Self {
        Self {
            normal: Vec2::new(1.5, 1.2).normalize() * 0.6,
            fast: Vec2::new(1.5, 1.2).normalize(),
            up: Vec2::new(0.0, 1.1),
            drop: Vec2::new(0.0, 0.0),
            lob: Vec2::new(1.0, 2.5).normalize() * 1.1,
            roll: Vec2::new(0.4, -0.1),
            spin: 0.0,
        }
    }
    /// `Self::standard` with the throw values multiplied by `strength`.
    fn strength(strength: f32) -> Self {
        Self {
            normal: Self::standard().normal * strength,
            fast: Self::standard().fast * strength,
            up: Self::standard().up * strength,
            drop: Self::standard().drop * strength,
            lob: Self::standard().lob * strength,
            roll: Self::standard().roll * strength,
            spin: 0.0,
        }
    }
    fn with_spin(self, spin: f32) -> Self {
        Self { spin, ..self }
    }
    /// Chooses one of the throw values based on a [`PlayerControl`]
    fn velocity_from_control(&self, player_control: &PlayerControl) -> Vec2 {
        let PlayerControl { move_direction, .. } = player_control;
        let y = move_direction.y;
        let moving = move_direction.x.abs() > 0.0;
        if y < 0.0 {
            if moving {
                return self.roll;
            } else {
                return self.drop;
            }
        }
        if moving {
            if y > 0.0 {
                self.lob
            } else {
                self.fast
            }
        } else if y > 0.0 {
            self.up
        } else {
            self.normal
        }
    }
}

fn throw_dropped_items(
    entities: Res<Entities>,
    item_throws: Comp<ItemThrow>,
    items: Comp<Item>,
    player_inputs: Res<PlayerInputs>,
    player_indexes: Comp<PlayerIdx>,
    mut items_dropped: CompMut<ItemDropped>,
    mut bodies: CompMut<KinematicBody>,
    mut attachments: CompMut<PlayerBodyAttachment>,
    mut sprites: CompMut<AtlasSprite>,
    mut commands: Commands,
) {
    for (entity, (_items, item_throw, body)) in
        entities.iter_with((&items, &item_throws, &mut bodies))
    {
        if let Some(ItemDropped { player }) = items_dropped.remove(entity) {
            commands.add(PlayerCommand::set_inventory(player, None));
            attachments.remove(entity);

            let player_sprite = sprites.get_mut(player).unwrap();

            let horizontal_flip_factor = if player_sprite.flip_x {
                Vec2::new(-1.0, 1.0)
            } else {
                Vec2::ONE
            };

            let throw_velocity = item_throw.velocity_from_control(
                &player_inputs
                    .players
                    .get(player_indexes.get(player).unwrap().0)
                    .unwrap()
                    .control,
            );

            body.velocity = throw_velocity * horizontal_flip_factor;
            body.angular_velocity =
                item_throw.spin * horizontal_flip_factor.x * throw_velocity.y.signum();

            body.is_deactivated = false;
        }
    }
}
