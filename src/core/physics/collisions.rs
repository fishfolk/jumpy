//! Collision detection implementation.

use std::hash::BuildHasherDefault;

use indexmap::IndexMap;

pub use rapier2d::prelude as rapier;
pub use shape::*;

mod shape;

use crate::impl_system_param;
use crate::prelude::*;

/// Resource containing the data structures needed for rapier collision detection.
#[derive(HasSchema, Default)]
pub struct RapierContext {
    pub collision_pipeline: rapier::CollisionPipeline,
    pub broad_phase: rapier::BroadPhase,
    pub narrow_phase: rapier::NarrowPhase,
    pub query_pipeline: rapier::QueryPipeline,
    pub collider_set: rapier::ColliderSet,
    pub rigid_body_set: rapier::RigidBodySet,
    pub collider_shape_cache: ColliderShapeCache,
    pub collision_cache: CollisionCache,
}

impl Clone for RapierContext {
    fn clone(&self) -> Self {
        Self {
            // The collision pipeline is just a cache and while we can't clone it, creating a new one is
            // perfectly valid.
            collision_pipeline: default(),
            broad_phase: self.broad_phase.clone(),
            narrow_phase: self.narrow_phase.clone(),
            query_pipeline: self.query_pipeline.clone(),
            collider_set: self.collider_set.clone(),
            rigid_body_set: self.rigid_body_set.clone(),
            collider_shape_cache: self.collider_shape_cache.clone(),
            collision_cache: self.collision_cache.clone(),
        }
    }
}

/// A cache containing a map of entities, to the list of entities that each entity is currently
/// intersecting with.
pub struct CollisionCache {
    /// The collisions in the cache.
    pub collisions: Arc<AtomicCell<IndexMap<Entity, Vec<Entity>, EntityBuildHasher>>>,
}

impl Default for CollisionCache {
    fn default() -> Self {
        Self {
            collisions: Arc::new(AtomicCell::new(IndexMap::default())),
        }
    }
}

/// Pass-through hasher for entities to reduce hashing cost when using them as keys in a hash map.
#[derive(Default)]
pub struct EntityHasher {
    bytes_so_far: usize,
    data: [u8; 8],
}
type EntityBuildHasher = BuildHasherDefault<EntityHasher>;

impl std::hash::Hasher for EntityHasher {
    fn finish(&self) -> u64 {
        u64::from_ne_bytes(self.data)
    }
    fn write(&mut self, bytes: &[u8]) {
        if self.bytes_so_far + bytes.len() <= 8 {
            self.data[self.bytes_so_far..(self.bytes_so_far + bytes.len())].copy_from_slice(bytes);
            self.bytes_so_far += bytes.len()
        } else {
            panic!("Too much data for `EntityHasher`. Will only accept 64 bits.")
        }
    }
}

impl CollisionCache {
    /// Get the set of entities that the given `entity` is intersecting.
    pub fn get(&self, entity: Entity) -> RefMut<'_, Vec<Entity>> {
        RefMut::map(self.collisions.borrow_mut(), |x| {
            x.entry(entity).or_default()
        })
    }
}

impl Clone for CollisionCache {
    fn clone(&self) -> Self {
        Self {
            collisions: Arc::new(AtomicCell::new((*self.collisions.borrow()).clone())),
        }
    }
}

/// Update the collision cache with rapier collision events.
impl rapier::EventHandler for &mut CollisionCache {
    fn handle_collision_event(
        &self,
        _bodies: &rapier::RigidBodySet,
        colliders: &rapier::ColliderSet,
        event: rapier::CollisionEvent,
        _contact_pair: Option<&rapier::ContactPair>,
    ) {
        match event {
            rapier::CollisionEvent::Started(a, b, _) => {
                let a_ent = RapierUserData::entity(colliders.get(a).unwrap().user_data);
                let b_ent = RapierUserData::entity(colliders.get(b).unwrap().user_data);

                self.collisions
                    .borrow_mut()
                    .entry(a_ent)
                    .or_default()
                    .push(b_ent);
                self.collisions
                    .borrow_mut()
                    .entry(b_ent)
                    .or_default()
                    .push(a_ent);
            }
            rapier::CollisionEvent::Stopped(a, b, _) => {
                let Some(a_ent) = colliders
                    .get(a)
                    .map(|x| RapierUserData::entity(x.user_data))
                else {
                    return;
                };
                let Some(b_ent) = colliders
                    .get(b)
                    .map(|x| RapierUserData::entity(x.user_data))
                else {
                    return;
                };

                self.collisions
                    .borrow_mut()
                    .entry(a_ent)
                    .or_default()
                    .retain(|e| e != &b_ent);
                self.collisions
                    .borrow_mut()
                    .entry(b_ent)
                    .or_default()
                    .retain(|e| e != &a_ent);
            }
        }
    }

    fn handle_contact_force_event(
        &self,
        _dt: rapier::Real,
        _bodies: &rapier::RigidBodySet,
        _colliders: &rapier::ColliderSet,
        _contact_pair: &rapier::ContactPair,
        _total_force_magnitude: rapier::Real,
    ) {
    }
}

impl_system_param! {
    pub struct CollisionWorld<'a> {
        entities: Res<'a, Entities>,

        /// The rapier context.
        ctx: ResMutInit<'a, RapierContext>,

        /// Actors are things like players that move around and detect collisions, but don't collide
        /// with other actors.
        actors: CompMut<'a, Actor>,
        /// Solids are things like walls and platforms, that aren't tiles, that have solid
        /// collisions.
        solids: CompMut<'a, Solid>,
        /// A collider is anything that can detect collisions in the world other than tiles, and
        /// must either be an [`Actor`] or [`Solid`] to participate in collision detection.
        colliders: CompMut<'a, Collider>,
        /// Contains the rapier collider handles for each map tile.
        tile_rapier_handles: CompMut<'a, TileRapierHandle>,

        tile_layers: Comp<'a, TileLayer>,
        tile_collision_kinds: Comp<'a, TileCollisionKind>,
        spawned_map_layer_metas: Comp<'a, SpawnedMapLayerMeta>,
    }
}

/// An actor in the physics simulation.
#[derive(Default, Clone, Copy, Debug, HasSchema)]
#[repr(C)]
pub struct Actor;

/// A solid in the physics simulation.
#[derive(Default, Clone, Copy, Debug, HasSchema)]
#[repr(C)]
pub struct Solid {
    pub disabled: bool,
    pub pos: Vec2,
    pub size: Vec2,
    #[schema(opaque)]
    pub rapier_handle: Option<rapier::RigidBodyHandle>,
}

/// A collider body in the physics simulation.
///
/// This is only used for actors in the simulation, not for tiles or solids.
#[derive(Default, Clone, Debug, HasSchema)]
#[repr(C)]
pub struct Collider {
    // TODO: We used to have an offset here in the `Collider` struct, but I think maybe that should
    // become a part of the collision shape, not part of the collider. So if you need an offset
    // collider, maybe that means a compound collider shape with an offset collider in it.
    //
    // When we have a separate offset here, we have to remember and correctly apply the offset,
    // every time we check a collision between this colliders shape, at the colliders transform. It
    // kept causing bugs in colliders with offsets. That may still be the best option, and we just
    // have to deal with it, but we should consider the offset being included in the collider shape.
    pub shape: ColliderShape,
    // Whether or not the collider is disabled.
    pub disabled: bool,
    /// Whether or not the collider wants to drop through jump-through platforms.
    pub descent: bool,
    /// Whether or not the collider is in the process of going through a jump-through platform.
    pub seen_wood: bool,
    /// The handle to the Rapier rigid body associated to this collider, if one has been spawned as
    /// of yet.
    #[schema(opaque)]
    pub rapier_handle: Option<rapier::RigidBodyHandle>,
}

/// Component added to tiles that have been given corresponding rapier colliders.
#[derive(Default, Clone, Debug, HasSchema, Deref, DerefMut)]
pub struct TileRapierHandle(pub rapier::RigidBodyHandle);

/// Namespace struct for converting rapier collider user data to/from [`Entity`].
pub struct RapierUserData;
impl RapierUserData {
    /// Create rapier user data value from the entity `e`.
    pub fn from(e: Entity) -> u128 {
        let mut out = 0u128;

        out |= e.index() as u128;
        out |= (e.generation() as u128) << 32;

        out
    }

    /// Get an [`Entity`] from the given Rapier user data ( assuming the user data was created with
    /// [`RapierUserData::from`] ).
    pub fn entity(user_data: u128) -> Entity {
        let index = (u32::MAX as u128) & user_data;
        let generation = (u32::MAX as u128) & (user_data >> 32);
        Entity::new(index as u32, generation as u32)
    }
}

/// The kind of collision that a map tile has.
#[derive(Default, PartialEq, Eq, Clone, Copy, Debug, HasSchema, Serialize, Deserialize)]
#[repr(u8)]
#[derive_type_data(SchemaDeserialize)]
pub enum TileCollisionKind {
    #[default]
    Empty,
    Solid,
    JumpThrough,
}

impl<'a> CollisionWorld<'a> {
    /// Updates the collision world with the entity's actual transforms.
    ///
    /// If the transform of an entity is changed without calling `update()`, then collision queries
    /// will be out-of-date with the actual entity positions.
    ///
    /// > **⚠️ Warning:** This does **not** update the map tile collisions. To do that, call
    /// > [`update_tiles()`][Self::update_tiles] instead.
    pub fn update<'b, Tq>(&mut self, transforms: Tq)
    where
        Tq: QueryItem,
        Tq::Iter: Iterator<Item = &'b Transform>,
    {
        puffin::profile_function!();

        self.sync_colliders(transforms);

        let RapierContext {
            broad_phase,
            collider_set,
            query_pipeline,
            collision_cache,
            rigid_body_set,
            narrow_phase,
            collision_pipeline,
            ..
        } = &mut *self.ctx;

        // Delete any bodies that don't have alive entities
        let mut to_delete = Vec::new();
        for (handle, body) in rigid_body_set.iter() {
            let entity = RapierUserData::entity(body.user_data);

            if !self.entities.is_alive(entity) {
                // Remove any collisions with the killed entity from the collision cache.
                let mut collisions = collision_cache.collisions.borrow_mut();
                let colliding_with = collisions.remove(&entity);
                if let Some(colliding_with) = colliding_with {
                    for other_entity in colliding_with {
                        if let Some(collisions) = collisions.get_mut(&other_entity) {
                            collisions.retain(|e| e != &entity);
                        }
                    }
                }

                // Delete the rigid body
                to_delete.push(handle);
            }
        }

        for body_handle in to_delete {
            rigid_body_set.remove(
                body_handle,
                &mut default(),
                collider_set,
                &mut default(),
                &mut default(),
                true,
            );
        }

        // Update the collision pipeline
        {
            puffin::profile_scope!("Collision Pipeline Step");
            collision_pipeline.step(
                0.0,
                broad_phase,
                narrow_phase,
                rigid_body_set,
                collider_set,
                None,
                &(),
                &collision_cache,
            );
        }

        // Update the query pipeline
        {
            puffin::profile_scope!("Query Pipeline Update");
            query_pipeline.update(rigid_body_set, collider_set);
        }
    }

    /// Sync the transforms and attributes ( like `disabled` ) of the colliders. ( Does not update
    /// collision pipeline, and is only for use internally. )
    fn sync_colliders<'b, Tq>(&mut self, transforms: Tq)
    where
        Tq: QueryItem,
        Tq::Iter: Iterator<Item = &'b Transform>,
    {
        puffin::profile_function!();

        let RapierContext {
            rigid_body_set,
            collider_set,
            collider_shape_cache,
            ..
        } = &mut *self.ctx;
        for (ent, (transform, collider)) in
            self.entities.iter_with((transforms, &mut self.colliders))
        {
            // Get the rapier shape.
            //
            // TODO: Evaluate whether or not caching the colliders like this actually improves
            // performance.
            let shared_shape = collider_shape_cache.shared_shape(collider.shape);

            // Get the handle to the rapier collider, creating it if it doesn't exist.
            let rapier_handle = collider.rapier_handle.get_or_insert_with(|| {
                let body_handle = rigid_body_set.insert(
                    rapier::RigidBodyBuilder::dynamic().user_data(RapierUserData::from(ent)),
                );
                collider_set.insert_with_parent(
                    rapier::ColliderBuilder::new(shared_shape.clone())
                        .active_events(rapier::ActiveEvents::COLLISION_EVENTS)
                        .active_collision_types(rapier::ActiveCollisionTypes::all())
                        .user_data(RapierUserData::from(ent)),
                    body_handle,
                    rigid_body_set,
                );
                body_handle
            });
            let rapier_body = rigid_body_set.get_mut(*rapier_handle).unwrap();

            // Set the transform of the collider.
            rapier_body.set_position(
                rapier::Isometry::new(
                    transform.translation.truncate().to_array().into(),
                    transform.rotation.to_euler(EulerRot::XYZ).2,
                ),
                true,
            );
            let rapier_collider = collider_set.get_mut(rapier_body.colliders()[0]).unwrap();
            rapier_collider.set_enabled(!collider.disabled);
            rapier_collider.set_position_wrt_parent(rapier::Isometry::new(default(), 0.0));
        }

        for (solid_ent, solid) in self.entities.iter_with(&mut self.solids) {
            let bones_shape = ColliderShape::Rectangle { size: solid.size };
            let shared_shape = collider_shape_cache.shared_shape(bones_shape);

            // Get or create a collider for the solid
            let handle = solid.rapier_handle.get_or_insert_with(|| {
                let body_handle = rigid_body_set.insert(
                    rapier::RigidBodyBuilder::fixed().user_data(RapierUserData::from(solid_ent)),
                );
                collider_set.insert_with_parent(
                    rapier::ColliderBuilder::new(shared_shape.clone())
                        .active_events(rapier::ActiveEvents::COLLISION_EVENTS)
                        .active_collision_types(rapier::ActiveCollisionTypes::all())
                        .user_data(RapierUserData::from(solid_ent)),
                    body_handle,
                    rigid_body_set,
                );
                body_handle
            });
            let solid_body = rigid_body_set.get_mut(*handle).unwrap();

            // Update the solid position
            solid_body.set_translation(rapier::Vector::new(solid.pos.x, solid.pos.y), false);

            let rapier_collider = collider_set.get_mut(solid_body.colliders()[0]).unwrap();
            rapier_collider.set_enabled(!solid.disabled);
            rapier_collider.set_position_wrt_parent(rapier::Isometry::new(default(), 0.0));
            rapier_collider.set_shape(shared_shape.clone());
        }
    }

    /// Update all of the map tile collisions.
    ///
    /// You should only need to call this when spawning or otherwise completely rebuilding the map
    /// layout.
    pub fn update_tiles(&mut self) {
        self.update_tiles_with_filter(|_, _| true);
    }

    /// Update the collision for the tile with the given layer index and map grid position.
    pub fn update_tile(&mut self, layer_idx: u32, pos: UVec2) {
        self.update_tiles_with_filter(|idx, p| layer_idx == idx && pos == p);
    }

    /// Update the collisions for map tiles that pass the given filter.
    ///
    /// The filter is a function that takes the layer index and the tile position as an argument.
    pub fn update_tiles_with_filter<F>(&mut self, mut filter: F)
    where
        F: FnMut(u32, UVec2) -> bool,
    {
        let RapierContext {
            rigid_body_set,
            collider_set,
            collider_shape_cache,
            ..
        } = &mut *self.ctx;
        for (_, (layer, meta)) in self
            .entities
            .iter_with((&self.tile_layers, &self.spawned_map_layer_metas))
        {
            let tile_shared_shape = collider_shape_cache.shared_shape(ColliderShape::Rectangle {
                size: layer.tile_size,
            });
            for x in 0..layer.grid_size.x {
                for y in 0..layer.grid_size.y {
                    let pos = uvec2(x, y);
                    if !filter(meta.layer_idx, pos) {
                        continue;
                    };

                    let Some(tile_ent) = layer.get(pos) else {
                        continue;
                    };
                    let collider_x = x as f32 * layer.tile_size.x + layer.tile_size.x / 2.0;
                    let collider_y = y as f32 * layer.tile_size.y + layer.tile_size.y / 2.0;

                    // Get or create a collider for the tile
                    let handle = self
                        .tile_rapier_handles
                        .get(tile_ent)
                        .map(|x| **x)
                        .unwrap_or_else(|| {
                            let body_handle = rigid_body_set.insert(
                                rapier::RigidBodyBuilder::fixed()
                                    .user_data(RapierUserData::from(tile_ent)),
                            );
                            collider_set.insert_with_parent(
                                rapier::ColliderBuilder::new(tile_shared_shape.clone())
                                    .active_events(rapier::ActiveEvents::COLLISION_EVENTS)
                                    .active_collision_types(rapier::ActiveCollisionTypes::all())
                                    .user_data(RapierUserData::from(tile_ent)),
                                body_handle,
                                rigid_body_set,
                            );
                            self.tile_rapier_handles
                                .insert(tile_ent, TileRapierHandle(body_handle));
                            body_handle
                        });
                    let tile_body = rigid_body_set.get_mut(handle).unwrap();

                    // Update the collider position
                    tile_body.set_translation(rapier::Vector::new(collider_x, collider_y), false);
                }
            }
        }
    }

    /// When spawning or teleporting an entity, this should be called to make sure the entity
    /// doesn't get stuck in semi-solid platforms, and properly falls out of them if it happens to
    /// be colliding with one when spawned.
    //
    // TODO: I believe we can make this method unnecessary by correctly detecting when a body is
    // stuck in a wood platform, with no ground below it.
    pub fn handle_teleport(&mut self, entity: Entity) {
        if self
            .ctx
            .collision_cache
            .get(entity)
            .iter()
            .any(|x| self.tile_collision_kinds.get(*x) == Some(&TileCollisionKind::JumpThrough))
        {
            let collider = self.colliders.get_mut(entity).unwrap();
            collider.descent = true;
            collider.seen_wood = true;
        }
    }

    /// Returns the collisions that one actor has with any other actors.
    pub fn actor_collisions(&self, entity: Entity) -> Vec<Entity> {
        if !self.actors.contains(entity) {
            return default();
        }
        if !self.colliders.contains(entity) {
            return default();
        };

        self.ctx
            .collision_cache
            .get(entity)
            .iter()
            .filter(|x| self.actors.contains(**x))
            .copied()
            .collect()
    }

    /// Returns the collisions that one actor has with any other actors filtered by the given Fn
    pub fn actor_collisions_filtered(
        &self,
        entity: Entity,
        filter: impl Fn(Entity) -> bool,
    ) -> Vec<Entity> {
        if !self.actors.contains(entity) {
            return default();
        }
        if !self.colliders.contains(entity) {
            return default();
        };

        self.ctx
            .collision_cache
            .get(entity)
            .iter()
            .filter(|x| self.actors.contains(**x) && filter(**x))
            .copied()
            .collect()
    }

    /// Put the entity's collider into descent mode so that it will fall through jump-through
    /// platforms.
    pub fn descent(&mut self, entity: Entity) {
        if self.actors.contains(entity) {
            let collider = self.colliders.get_mut(entity).unwrap();
            collider.descent = true;
        }
    }

    /// Attempt to move a body vertically. This will return `true` if an obstacle was run into that
    /// caused the movement to stop short.
    pub fn move_vertical(
        &mut self,
        transforms: &mut CompMut<Transform>,
        entity: Entity,
        mut dy: f32,
    ) -> bool {
        puffin::profile_function!();

        let RapierContext {
            query_pipeline,
            collider_set,
            rigid_body_set,
            collider_shape_cache,
            ..
        } = &mut *self.ctx;
        assert!(self.actors.contains(entity));
        if dy == 0.0 {
            return false;
        }

        // Get the shape and position info for the given entity
        let collider = self.colliders.get_mut(entity).unwrap();
        let transform = *transforms.get(entity).unwrap();
        let mut position = rapier::Isometry::new(
            transform.translation.truncate().into(),
            transform.rotation.to_euler(EulerRot::XYZ).2,
        );
        let shape = collider_shape_cache.shared_shape(collider.shape);

        let mut movement = 0.0;
        let collided = loop {
            // Do a shape cast in the direction of movement
            let velocity = rapier::Vector::new(0.0, dy);
            let collision = query_pipeline.cast_shape(
                rigid_body_set,
                collider_set,
                &position,
                &velocity,
                &**shape,
                1.0,
                true,
                rapier::QueryFilter::new().predicate(&|_handle, rapier_collider| {
                    let ent = RapierUserData::entity(rapier_collider.user_data);

                    if self.solids.contains(ent) {
                        // Include all solid collisions
                        return true;
                    }

                    let Some(tile_kind) = self.tile_collision_kinds.get(ent) else {
                        // Ignore non-tile collisions
                        return false;
                    };

                    // Ignore jump-through tiles if we have already seen wood
                    !(collider.seen_wood && *tile_kind == TileCollisionKind::JumpThrough)
                }),
            );

            if let Some((handle, toi)) = collision {
                let ent = RapierUserData::entity(collider_set.get(handle).unwrap().user_data);

                // Move up to the point of collision
                let diff = dy * toi.toi;
                movement += diff;
                position.translation.y += diff;

                // Subtract from the remaining attempted movement
                dy -= diff;

                if self.solids.contains(ent) {
                    break true;
                }

                let tile_kind = *self.tile_collision_kinds.get(ent).unwrap();

                // collider wants to go down and collided with jumpthrough tile
                if tile_kind == TileCollisionKind::JumpThrough && collider.descent {
                    collider.seen_wood = true;
                }
                // collider wants to go up and encoutered jumpthrough obstace
                if tile_kind == TileCollisionKind::JumpThrough && dy > 0.0 {
                    collider.seen_wood = true;
                    collider.descent = true;
                }

                // If we hit a solid block, or a jumpthrough tile that we aren't falling through
                if !(tile_kind == TileCollisionKind::JumpThrough
                    && (collider.descent || dy > 0.0 || collider.seen_wood))
                {
                    // Indicate we ran into something and stop processing
                    break true;
                }

            // If there is no collision
            } else {
                movement += dy;
                // Indicate we didn't run into anything and stop processing
                break false;
            }
        };

        // Move the entity
        let transform = transforms.get_mut(entity).unwrap();
        transform.translation.y += movement - if collided { 0.1 * dy.signum() } else { 0.0 };

        // Final check, if we are out of woods after the move - reset wood flags
        {
            puffin::profile_scope!("out of woods check");
            let is_in_jump_through = query_pipeline
                .intersection_with_shape(
                    rigid_body_set,
                    collider_set,
                    &(
                        transform.translation.truncate(),
                        transform.rotation.to_euler(EulerRot::XYZ).2,
                    )
                        .into(),
                    &**shape,
                    rapier::QueryFilter::new().predicate(&|_handle, collider| {
                        let ent = RapierUserData::entity(collider.user_data);
                        self.tile_collision_kinds.get(ent) == Some(&TileCollisionKind::JumpThrough)
                    }),
                )
                .is_some();

            if !is_in_jump_through {
                collider.seen_wood = false;
                collider.descent = false;
            }
        }

        collided
    }

    /// Attempt to move a body horizontally. This will return `true` if an obstacle was run into
    /// that caused the movement to stop short.
    pub fn move_horizontal(
        &mut self,
        transforms: &mut CompMut<Transform>,
        entity: Entity,
        mut dx: f32,
    ) -> bool {
        puffin::profile_function!();

        let RapierContext {
            query_pipeline,
            collider_set,
            rigid_body_set,
            collider_shape_cache,
            ..
        } = &mut *self.ctx;
        assert!(self.actors.contains(entity));
        if dx == 0.0 {
            return false;
        }

        // Get the shape and position info for the given entity
        let collider = self.colliders.get_mut(entity).unwrap();
        let transform = *transforms.get(entity).unwrap();
        let mut position = (
            transform.translation.truncate(),
            transform.rotation.to_euler(EulerRot::XYZ).2,
        )
            .into();
        let shape = collider_shape_cache.shared_shape(collider.shape);

        let mut movement = 0.0;
        let collided = 'collision: loop {
            // Do a shape cast in the direction of movement
            let velocity = rapier::Vector::new(dx, 0.0);
            let collision = {
                puffin::profile_scope!("cast shape");
                query_pipeline.cast_shape(
                    rigid_body_set,
                    collider_set,
                    &position,
                    &velocity,
                    &**shape,
                    1.0,
                    true,
                    rapier::QueryFilter::new().predicate(&|_handle, rapier_collider| {
                        let ent = RapierUserData::entity(rapier_collider.user_data);

                        if self.solids.contains(ent) {
                            // Include all solid collisions
                            return true;
                        }

                        let Some(tile_kind) = self.tile_collision_kinds.get(ent) else {
                            // Ignore non-tile collisions
                            return false;
                        };

                        // Ignore jump-through tiles if we have already seen wood.
                        !(collider.seen_wood && *tile_kind == TileCollisionKind::JumpThrough)
                    }),
                )
            };

            if let Some((handle, toi)) = collision {
                let ent = RapierUserData::entity(collider_set.get(handle).unwrap().user_data);

                // Move up to the point of collision
                let diff = dx * toi.toi;
                movement += diff;
                position.translation.x += diff;

                // Subtract from the remaining attempted movement
                dx -= diff;

                if self.solids.contains(ent) {
                    break true;
                }

                let tile_kind = *self.tile_collision_kinds.get(ent).unwrap();

                // If we ran into a jump-through tile, go through it and continue casting
                if tile_kind == TileCollisionKind::JumpThrough {
                    collider.seen_wood = true;
                    collider.descent = true;

                // If we ran into any other kind of tile
                } else {
                    // Indicate we ran into something and stop processing
                    break 'collision true;
                }

            // If there is no collision
            } else {
                movement += dx;
                // Indicate we didn't run into anything and stop processing
                break 'collision false;
            }
        };

        // Move the entity
        let transform = transforms.get_mut(entity).unwrap();
        transform.translation.x += movement - if collided { 0.1 * dx.signum() } else { 0.0 };

        // Final check, if we are out of woods after the move - reset wood flags
        {
            puffin::profile_scope!("out of woods check");
            let is_in_jump_through = query_pipeline
                .intersection_with_shape(
                    rigid_body_set,
                    collider_set,
                    &(
                        transform.translation.truncate(),
                        transform.rotation.to_euler(EulerRot::XYZ).2,
                    )
                        .into(),
                    &**shape,
                    rapier::QueryFilter::new().predicate(&|_handle, collider| {
                        let ent = RapierUserData::entity(collider.user_data);
                        self.tile_collision_kinds.get(ent) == Some(&TileCollisionKind::JumpThrough)
                    }),
                )
                .is_some();

            if !is_in_jump_through {
                collider.seen_wood = false;
                collider.descent = false;
            }
        }

        collided
    }

    /// Returns whether or not there is a tile or solid at the given position.
    ///
    /// > ⚠️ **Warning:** There is a slight difference to how `tile_collision_point` and
    /// > [`tile_collision`][Self::tile_collision] reports collisions.
    /// >
    /// > [`tile_collision`][Self::tile_collision] will report a collision if the collider shape is
    /// > perfectly lined up along the edge of a tile, but `tile_collision_point` won't.
    #[allow(unused)]
    pub fn solid_at(&self, pos: Vec2) -> bool {
        self.solid_collision_point(pos)
            || self.tile_collision_point(pos) == TileCollisionKind::Solid
    }

    pub fn solid_collision_point(&self, pos: Vec2) -> bool {
        for (_, (solid, collider)) in self.entities.iter_with((&self.solids, &self.colliders)) {
            let bbox = collider
                .shape
                .bounding_box(Transform::from_translation(solid.pos.extend(0.0)));
            if bbox.contains(pos) {
                return true;
            }
        }

        false
    }

    /// Returns the tile collision at the given point.
    ///
    /// > ⚠️ **Warning:** There is a slight difference to how `tile_collision_point` and
    /// > [`tile_collision`][Self::tile_collision] reports collisions.
    /// >
    /// > [`tile_collision`][Self::tile_collision] will report a collision if the collider shape is
    /// > perfectly lined up along the edge of a tile, but `tile_collision_point` won't.
    #[allow(unused)]
    pub fn tile_collision_point(&self, pos: Vec2) -> TileCollisionKind {
        for (entity, tile_layer) in self.entities.iter_with(&self.tile_layers) {
            let TileLayer { tile_size, .. } = tile_layer;

            let x = (pos.x / tile_size.y).floor() as u32;
            let y = (pos.y / tile_size.x).floor() as u32;
            let tile_entity = tile_layer.get(UVec2::new(x, y));
            if let Some(tile_entity) = tile_entity {
                return self
                    .tile_collision_kinds
                    .get(tile_entity)
                    .copied()
                    .unwrap_or_default();
            }
        }

        TileCollisionKind::Empty
    }

    /// Get the [`TileCollisionKind`] of the first tile detected colliding with the `shape` at the
    /// given `transform`.
    pub fn tile_collision(&self, transform: Transform, shape: ColliderShape) -> TileCollisionKind {
        self.tile_collision_filtered(transform, shape, |_| true)
    }

    pub fn tile_collision_filtered(
        &self,
        transform: Transform,
        shape: ColliderShape,
        filter: impl Fn(Entity) -> bool,
    ) -> TileCollisionKind {
        self.ctx
            .query_pipeline
            .intersection_with_shape(
                &self.ctx.rigid_body_set,
                &self.ctx.collider_set,
                &(
                    transform.translation.truncate(),
                    transform.rotation.to_euler(EulerRot::XYZ).2,
                )
                    .into(),
                &*shape.shared_shape(),
                rapier::QueryFilter::new().predicate(&|_handle, collider| {
                    let ent = RapierUserData::entity(collider.user_data);
                    (self.solids.contains(ent) || self.tile_collision_kinds.contains(ent))
                        && filter(ent)
                }),
            )
            .map(|x| RapierUserData::entity(self.ctx.collider_set.get(x).unwrap().user_data))
            .and_then(|ent| {
                if self.solids.contains(ent) {
                    return Some(TileCollisionKind::Solid);
                }
                self.tile_collision_kinds.get(ent).copied()
            })
            .unwrap_or_default()
    }

    /// Get the collider for the given entity.
    pub fn get_collider(&self, actor: Entity) -> &Collider {
        assert!(self.actors.contains(actor));
        self.colliders.get(actor).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn convert_entity_to_from_user_data() {
        let e1 = Entity::new(102395950, 10394875);
        let bits = RapierUserData::from(e1);
        let e2 = RapierUserData::entity(bits);
        assert_eq!(e1, e2);
    }
}
