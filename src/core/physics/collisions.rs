//! Collision detection implementation.

use std::hash::BuildHasherDefault;

use indexmap::IndexMap;

use rapier::Vector;
use rapier2d::geometry::InteractionGroups;
pub use rapier2d::prelude as rapier;
pub use shape::*;

pub mod filtering;
mod shape;

use crate::collisions::filtering::CollisionGroup;
use crate::collisions::filtering::SolverGroup;
use crate::impl_system_param;
use crate::prelude::*;

/// Resource containing the data structures needed for rapier collision detection.
#[derive(HasSchema, Default)]
pub struct RapierContext {
    pub collision_pipeline: rapier::CollisionPipeline,
    pub broad_phase: rapier::DefaultBroadPhase,
    pub narrow_phase: rapier::NarrowPhase,
    pub query_pipeline: rapier::QueryPipeline,
    pub collider_set: rapier::ColliderSet,
    pub rigid_body_set: rapier::RigidBodySet,
    pub collider_shape_cache: ColliderShapeCache,
    pub collision_cache: CollisionCache,
    pub physics_hooks: PhysicsHooks,
    pub physics_pipeline: rapier::PhysicsPipeline,
    pub islands: rapier::IslandManager,
    pub impulse_joints: rapier::ImpulseJointSet,
    pub ccd_solver: rapier::CCDSolver,
    pub multibody_joints: rapier::MultibodyJointSet,
    pub integration_params: rapier::IntegrationParameters,
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
            physics_hooks: self.physics_hooks.clone(),
            // Probably should keep this around, it's safe to drop data as only temp buffers,
            // but re-creating may hurt performance.
            physics_pipeline: rapier::PhysicsPipeline::default(),
            islands: self.islands.clone(),
            impulse_joints: self.impulse_joints.clone(),
            ccd_solver: self.ccd_solver.clone(),
            multibody_joints: self.multibody_joints.clone(),
            integration_params: self.integration_params,
        }
    }
}

/// A cache containing a map of entities, to the list of entities that each entity is currently
/// intersecting with.
pub struct CollisionCache {
    /// The collisions in the cache.
    pub collisions: Arc<AtomicCell<IndexMap<Entity, Vec<Entity>, EntityBuildHasher>>>,

    /// Map removed collider handles to entities. When Rapier gives collision event for removal,
    /// the collider is no longer in [`rapier::ColliderSet`], thus we cannot retrieve our user data
    /// and determine entity to be removed.
    ///
    /// Colliders should be added here on removal for event processing, this is cleared each frame.
    ///
    /// TODO: Consider a safer way to handle this that doesn't involve remembering to update this on
    /// removal.
    removed_colliders: IndexMap<rapier::ColliderHandle, Entity>,
}

impl Default for CollisionCache {
    fn default() -> Self {
        Self {
            collisions: Arc::new(AtomicCell::new(IndexMap::default())),
            removed_colliders: Default::default(),
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

    /// Notify cache of removal of collider to correctly handle stop event on removed collider.
    /// see `Self::removed_colliders_userdata` field comment for details.
    pub fn collider_removed(&mut self, entity: Entity, collider_handle: rapier::ColliderHandle) {
        self.removed_colliders.insert(collider_handle, entity);
    }

    /// Clear tracked data for removed colliders this frame (call after rapier update)
    pub fn clear_removed_colliders(&mut self) {
        self.removed_colliders.clear();
    }
}

impl Clone for CollisionCache {
    fn clone(&self) -> Self {
        Self {
            collisions: Arc::new(AtomicCell::new((*self.collisions.borrow()).clone())),
            removed_colliders: self.removed_colliders.clone(),
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
                let a_ent = match colliders.get(a) {
                    Some(a) => RapierUserData::entity(a.user_data),
                    None => {
                        if let Some(a_ent) = self.removed_colliders.get(&a) {
                            *a_ent
                        } else {
                            return;
                        }
                    }
                };
                let b_ent = match colliders.get(b) {
                    Some(b) => RapierUserData::entity(b.user_data),
                    None => {
                        if let Some(b_ent) = self.removed_colliders.get(&b) {
                            *b_ent
                        } else {
                            return;
                        }
                    }
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

/// Errors produced from physics system
#[derive(thiserror::Error, Debug)]
pub enum PhysicsError {
    #[error("Physics body not initialized: {0}")]
    BodyNotInitialized(String),
}

#[derive(Default, Clone)]
pub struct PhysicsHooks;

impl rapier::PhysicsHooks for PhysicsHooks {
    fn filter_contact_pair(
        &self,
        _context: &rapier::PairFilterContext,
    ) -> Option<rapier::SolverFlags> {
        // No contact pair filtering hook currently implemented
        Some(rapier::SolverFlags::COMPUTE_IMPULSES)
    }

    fn filter_intersection_pair(&self, _context: &rapier::PairFilterContext) -> bool {
        // No intersection pair hook currently implemented
        true
    }

    fn modify_solver_contacts(&self, context: &mut rapier::ContactModificationContext) {
        // Determine if jump through modifiation is needed:
        // (If one body is a player, and other is jump through tile.)

        let collider1 = context.colliders.get(context.collider1).unwrap();
        let body1 = context.bodies.get(context.rigid_body1.unwrap()).unwrap();
        let collider2 = context.colliders.get(context.collider2).unwrap();
        let body2 = context.bodies.get(context.rigid_body2.unwrap()).unwrap();

        let mut jump_through_body: Option<&rapier::RigidBody> = None;
        let mut other_body: Option<&rapier::RigidBody> = None;

        // Determine which body is jump through collider, if any.
        if collider1
            .solver_groups()
            .memberships
            .intersects(SolverGroup::JUMP_THROUGH.bits().into())
        {
            jump_through_body = Some(body1);
            other_body = Some(body2);
        } else if collider2
            .solver_groups()
            .memberships
            .intersects(SolverGroup::JUMP_THROUGH.bits().into())
        {
            jump_through_body = Some(body2);
            other_body = Some(body1);
        }

        if jump_through_body.is_some() {
            let other_body = other_body.unwrap();

            if other_body.linvel().y > 0.0 {
                context.solver_contacts.clear();
            }
        }
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
        tile_dynamic_colliders: Comp<'a, TileDynamicCollider>,
        spawned_map_layer_metas: Comp<'a, SpawnedMapLayerMeta>,
    }
}

impl<'a> CollisionWorld<'a> {
    /// Update shape of actor's [`Collider`]. Warns if entity does not have an [`Actor`] component.
    ///
    /// Updates shape on `Collider` and rebuilds rapier's collider on rigidbody.
    ///
    /// Use [`Self::set_actor_shape_from_builder`] for more control over new collider's settings.
    pub fn set_actor_shape(&mut self, entity: Entity, shape: ColliderShape) {
        let shared_shape = self.ctx.collider_shape_cache.shared_shape(shape);
        let new_collider = build_actor_rapier_collider(entity, shared_shape.clone());
        self.set_actor_shape_from_builder(entity, new_collider, shape);
    }

    /// Update shape of actor's [`Collider`]. Warns if entity does not have an [`Actor`] component.
    ///
    /// Updates shape on `Collider` and rebuilds rapier's collider on rigidbody.
    ///
    /// Accepts a [`rapier::ColliderBuilder`] (can get one with [`build_actor_rapier_collider`]) so
    /// other settings may be configured on new collider.
    pub fn set_actor_shape_from_builder(
        &mut self,
        entity: Entity,
        mut collider_builder: rapier::ColliderBuilder,
        shape: ColliderShape,
    ) {
        if !self.actors.contains(entity) {
            // This doesn't technically need be restricted, however we use default settings of collider for Actor,
            // and function would need to be updated to do this correctly for Solids, Tiles, or other classes of body.
            warn!("CollisionWorld::set_actor_shape called on entity that is not an Actor.");
            return;
        }

        if let Some(collider) = self.colliders.get_mut(entity) {
            collider.shape = shape;

            if let Some(handle) = collider.rapier_handle {
                let RapierContext {
                    rigid_body_set,
                    collision_cache,
                    collider_set,
                    islands,
                    ..
                } = &mut *self.ctx;

                let rapier_body = rigid_body_set.get_mut(handle).unwrap();

                {
                    let collider_handle = rapier_body.colliders()[0];
                    let current_rapier_collider = collider_set.get(collider_handle).unwrap();

                    // Update new collider with any settings that need to be synchronized
                    collider_builder = collider_builder.sensor(current_rapier_collider.is_sensor());

                    // Remove body's current collider
                    let wake_up = true;
                    collider_set.remove(collider_handle, islands, rigid_body_set, wake_up);

                    // Notify collision event cache handle was removed
                    //
                    // We may get a stop/start even while changing collider. This is required to make sure
                    // collision event is properly handled by rapier.
                    collision_cache.collider_removed(entity, collider_handle);
                }

                // Insert body's new collider
                collider_set.insert_with_parent(collider_builder, handle, rigid_body_set);
            } else {
                // We have an existing Collider but no rapier body yet, shape was updated on Collider
                // and will be used when body is created.
            }
        } else {
            // No existing collider, insert new one with shape.
            // Not really expecting this case to be called, but might as well handle it.
            // rapier body will be created on next call to `sync_colliders`.
            self.colliders.insert(
                entity,
                Collider {
                    shape,
                    ..Default::default()
                },
            );
        }
    }

    /// Call closure with mutable reference to [`rapier::RigidBody`] for entity.
    ///
    /// # Errors
    /// - [`PhysicsError::BodyNotInitialized`] when called before physics is updated to initialize body after
    ///   a [`Collider`] component is newly added. (If missing collider or rapier handle does not map to body).
    pub fn mutate_rigidbody(
        &mut self,
        entity: Entity,
        command: impl FnOnce(&mut rapier::RigidBody),
    ) -> Result<(), PhysicsError> {
        if let Some(collider) = self.colliders.get(entity) {
            if let Some(handle) = collider.rapier_handle {
                if let Some(body) = self.ctx.rigid_body_set.get_mut(handle) {
                    command(body);
                    Ok(())
                } else {
                    Err(PhysicsError::BodyNotInitialized(
                        "Rigidbody handle found but not in rigid body set.".to_string(),
                    ))
                }
            } else {
                Err(PhysicsError::BodyNotInitialized(
                    "Entity has collider that is missing rapier handle.".to_string(),
                ))
            }
        } else {
            Err(PhysicsError::BodyNotInitialized(
                "Entity does not have a Collider component.".to_string(),
            ))
        }
    }
}

/// Helper function for configuring ColliderBuilder for actors.
pub fn build_actor_rapier_collider(
    entity: Entity,
    shared_shape: rapier::SharedShape,
) -> rapier::ColliderBuilder {
    // Do not filter collision pairs, get all collision events
    let collision_membership = CollisionGroup::DEFAULT;
    let collision_filter = CollisionGroup::ALL;
    // Only generate contact forces (when simulating) with solids/tiles, not other dynamics.
    // This is not relevant if only Kinematic, only relevant if DynamicBody is added and switched to simulating.
    let simulation_membership = SolverGroup::DYNAMIC;
    let simulation_filter = SolverGroup::SOLID_WORLD | SolverGroup::JUMP_THROUGH;

    rapier::ColliderBuilder::new(shared_shape)
        .active_events(rapier::ActiveEvents::COLLISION_EVENTS)
        .active_collision_types(rapier::ActiveCollisionTypes::all())
        .collision_groups(rapier::InteractionGroups::new(
            collision_membership.bits().into(),
            collision_filter.bits().into(),
        ))
        .solver_groups(rapier::InteractionGroups::new(
            simulation_membership.bits().into(),
            simulation_filter.bits().into(),
        ))
        .sensor(true)
        .user_data(RapierUserData::from(entity))
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

/// Component added to tiles that have an additional collider used for interaction with
/// dynamic bodies that simulate physics.
///
/// This collider is added to rapier body stored in [`TileRapierHandle`]. If present,
/// default collider will not interact with dynamics, and this one will.
///
/// This is mostly useful for Jump through tiles.
#[derive(Default, Clone, HasSchema)]
pub struct TileDynamicCollider {
    /// Shape of collider, should be contained within tile.
    pub shape: ColliderShape,

    /// Offset of collider from center of tile.
    pub offset: Vec2,
}

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

impl TileCollisionKind {
    /// Get the solver group for tile, this is collision filtering group
    /// used against dynamic bodies that simulate physical collision.
    /// Does not impact event filtering.
    pub fn simulation_group_membership(&self) -> SolverGroup {
        match self {
            TileCollisionKind::Empty => SolverGroup::NONE,
            TileCollisionKind::Solid => SolverGroup::SOLID_WORLD,
            TileCollisionKind::JumpThrough => SolverGroup::JUMP_THROUGH,
        }
    }
}

/// Parameters for physics step
pub struct PhysicsParams {
    /// Gravity (positive value is downward force)
    pub gravity: f32,

    /// Terminal velocity (effectively min velocity on y axis, body will not fall faster than this).
    pub terminal_velocity: Option<f32>,
}

impl<'a> CollisionWorld<'a> {
    /// Updates the collision world with the entity's actual transforms.
    /// Advance physics, synchronize position of dynamic bodies.
    ///
    /// If the transform of an entity is changed without calling `update()`, then collision queries
    /// will be out-of-date with the actual entity positions.
    ///
    /// > **⚠️ Warning:** This does **not** update the map tile collisions. To do that, call
    /// > [`update_tiles()`][Self::update_tiles] instead.
    pub fn update(
        &mut self,
        dt: f32,
        physics_params: PhysicsParams,
        transforms: &mut CompMut<Transform>,
        dynamic_bodies: &mut CompMut<DynamicBody>,
    ) {
        puffin::profile_function!();

        self.sync_bodies(&*transforms, dynamic_bodies);
        self.apply_simulation_commands(&mut *dynamic_bodies);

        let RapierContext {
            broad_phase,
            collider_set,
            query_pipeline,
            collision_cache,
            rigid_body_set,
            narrow_phase,
            physics_hooks,
            physics_pipeline,
            islands,
            impulse_joints,
            ccd_solver,
            multibody_joints,
            integration_params,
            ..
        } = &mut *self.ctx;

        // Delete any bodies that don't have alive entities
        let mut to_delete = Vec::new();
        for (handle, body) in rigid_body_set.iter() {
            let entity = RapierUserData::entity(body.user_data);

            if !self.entities.is_alive(entity) {
                // Remove any collisions with the killed entity from the collision cache.
                let mut collisions = collision_cache.collisions.borrow_mut();
                let colliding_with = collisions.swap_remove(&entity);
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
                islands,
                collider_set,
                impulse_joints,
                multibody_joints,
                true,
            );
        }

        // Step physics pipeline, also steps collision pipeline and updates collision cache.
        integration_params.dt = dt;
        physics_pipeline.step(
            &Vector::new(0.0, -physics_params.gravity),
            integration_params,
            islands,
            broad_phase,
            narrow_phase,
            rigid_body_set,
            collider_set,
            impulse_joints,
            multibody_joints,
            ccd_solver,
            Some(query_pipeline),
            physics_hooks,
            &collision_cache,
        );

        // Iter on each dynamic rigid-bodies that moved.
        for rigid_body_handle in islands.active_dynamic_bodies() {
            let rigid_body = rigid_body_set.get_mut(*rigid_body_handle).unwrap();
            let entity = RapierUserData::entity(rigid_body.user_data);
            if let Some(dynamic_body) = dynamic_bodies.get_mut(entity) {
                if dynamic_body.is_dynamic {
                    let transform = transforms.get_mut(entity).unwrap();
                    let rotation = Quat::from_rotation_z(rigid_body.rotation().angle());
                    // Get translation from physics and preserve Z offset of transform
                    let translation: Vec3 = rigid_body
                        .translation()
                        .xy()
                        .push(transform.translation.z)
                        .into();
                    transform.translation = translation;
                    transform.rotation = rotation;

                    // Apply terminal velocity. Without this kinematic and dynamic objects may
                    // fall at different speeds even if same graviy applied. This is used to mirror
                    // kinematic motion of gravity + terminal vel limit.
                    if let Some(terminal_vel) = physics_params.terminal_velocity {
                        let mut vel = *rigid_body.linvel();
                        if vel.y < -terminal_vel {
                            vel.y = -terminal_vel;
                            rigid_body.set_linvel(vel, true);
                        }
                    }

                    // Update simulation output for tracking if transform is modified outside of rapier
                    // next frame.
                    dynamic_body.update_last_rapier_synced_transform(translation, rotation)
                } else {
                    warn!("Active dynamic bodies contained DynamicBody that is not simulating");
                }
            } else {
                warn!("Active dynamic bodies contained entity that does not have a DynamicBody.");
            }
        }

        // Reset tracking of removed colliders for this frame
        collision_cache.clear_removed_colliders();
    }

    /// Sync the transforms and attributes ( like `disabled` ) of the colliders.
    /// Creates rapier bodies for any object with collision.
    ///
    /// Handle [`DynamicBody`] toggling between simulation and kinematic mode.
    pub fn sync_bodies<'b, Tq>(&mut self, transforms: Tq, dynamic_bodies: &mut CompMut<DynamicBody>)
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
        for (ent, (transform, collider, dynamic_body)) in self.entities.iter_with((
            transforms,
            &mut self.colliders,
            &mut OptionalMut(dynamic_bodies),
        )) {
            // Get the rapier shape.
            //
            // TODO: Evaluate whether or not caching the colliders like this actually improves
            // performance.
            let shared_shape = collider_shape_cache.shared_shape(collider.shape);

            let is_dynamic = match dynamic_body.as_ref() {
                Some(dynamic_body) => dynamic_body.is_dynamic,
                None => false,
            };

            // Get the handle to the rapier collider, creating it if it doesn't exist.
            let rapier_handle = collider.rapier_handle.get_or_insert_with(|| {
                // Initialize body
                let body_handle = rigid_body_set.insert(if is_dynamic {
                    rapier::RigidBodyBuilder::dynamic().user_data(RapierUserData::from(ent))
                } else if KINEMATIC_MODE == rapier::RigidBodyType::KinematicPositionBased {
                    rapier::RigidBodyBuilder::kinematic_position_based()
                        .user_data(RapierUserData::from(ent))
                } else {
                    rapier::RigidBodyBuilder::kinematic_velocity_based()
                        .user_data(RapierUserData::from(ent))
                });

                collider_set.insert_with_parent(
                    build_actor_rapier_collider(ent, shared_shape.clone()),
                    body_handle,
                    rigid_body_set,
                );
                body_handle
            });
            let rapier_body = rigid_body_set.get_mut(*rapier_handle).unwrap();
            let rapier_collider = collider_set.get_mut(rapier_body.colliders()[0]).unwrap();

            if let Some(dynamic_body) = dynamic_body {
                // Handle changes in is_dynamic
                let was_dynamic = matches!(rapier_body.body_type(), rapier::RigidBodyType::Dynamic);
                if !was_dynamic && is_dynamic {
                    rapier_body.set_body_type(rapier::RigidBodyType::Dynamic, true);

                    // Clear any velocity that may be left from previously simulating body.
                    // If Dynamic is newly initialized or user wants to apply velocity changes before next step,
                    // `DynamicBody::push_simulation_command` may be used which is called after this operation.
                    rapier_body.set_linvel(Vector::zeros(), true);
                    rapier_body.set_angvel(0.0, true);

                    // TODO: We may want to synchronize kinematic body's gravity, mass, and other properties.

                    rapier_collider.set_sensor(false);

                    // Enable contact modification for all bodies to handle stuff like jump through.
                    rapier_collider.set_active_hooks(rapier::ActiveHooks::MODIFY_SOLVER_CONTACTS);
                } else if was_dynamic && !is_dynamic {
                    rapier_collider.set_sensor(true);
                    rapier_collider.set_active_hooks(rapier::ActiveHooks::empty());

                    rapier_body.set_body_type(KINEMATIC_MODE, true);
                }

                // This function still calls for position update if is_dynamic = false
                if dynamic_body.simulation_transform_needs_update(transform) {
                    rapier_body.set_position(
                        rapier::Isometry::new(
                            transform.translation.truncate().to_array().into(),
                            transform.rotation.to_euler(EulerRot::XYZ).2,
                        ),
                        true,
                    )
                }
            } else {
                // update position of kinematics
                //
                // TODO: we may want to use rapier::RigidBody::set_next_kinematic_position
                // so rapier computes velocity of kinematics for better interaction with dynamics,
                // however we don't currently have any kinematic <-> dynamic interaction.
                rapier_body.set_position(
                    rapier::Isometry::new(
                        transform.translation.truncate().to_array().into(),
                        transform.rotation.to_euler(EulerRot::XYZ).2,
                    ),
                    true,
                );
            }
            rapier_collider.set_enabled(!collider.disabled);
        }

        for (solid_ent, solid) in self.entities.iter_with(&mut self.solids) {
            let bones_shape = ColliderShape::Rectangle { size: solid.size };
            let shared_shape = collider_shape_cache.shared_shape(bones_shape);

            // Get or create a collider for the solid
            let handle = solid.rapier_handle.get_or_insert_with(|| {
                let body_handle = rigid_body_set.insert(
                    rapier::RigidBodyBuilder::fixed().user_data(RapierUserData::from(solid_ent)),
                );
                // Membership default, does not filter out collision events.
                let collision_membership = CollisionGroup::DEFAULT;
                let collision_filter = CollisionGroup::ALL;
                // Solids do not filter contact forces with other bodies.
                let simulation_membership = SolverGroup::SOLID_WORLD;
                let simulation_filter = SolverGroup::ALL;
                collider_set.insert_with_parent(
                    rapier::ColliderBuilder::new(shared_shape.clone())
                        .active_events(rapier::ActiveEvents::COLLISION_EVENTS)
                        .active_collision_types(rapier::ActiveCollisionTypes::all())
                        .collision_groups(InteractionGroups::new(
                            collision_membership.bits().into(),
                            collision_filter.bits().into(),
                        ))
                        .solver_groups(InteractionGroups::new(
                            simulation_membership.bits().into(),
                            simulation_filter.bits().into(),
                        ))
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

    /// Apply simulation commands to dynamic bodies.
    ///
    /// # Panics
    ///
    /// This should be called after bodies are initialized, [`DynamicBody`] must have
    /// a [`Collider`] with valid `rapier::RigidBodyHandle` otherwise will panic.
    fn apply_simulation_commands<'b, Dq>(&mut self, dynamic_bodies: Dq)
    where
        Dq: QueryItem,
        Dq::Iter: Iterator<Item = &'b mut DynamicBody>,
    {
        for (ent, dynamic_body) in self.entities.iter_with(dynamic_bodies) {
            // This will consume commands even if body is_dynamic is false.
            let commands = dynamic_body.simulation_commands();
            if dynamic_body.is_dynamic {
                let collider = self.colliders.get(ent).unwrap();
                let rapier_handle = collider.rapier_handle.unwrap();
                let rapier_body = self.ctx.rigid_body_set.get_mut(rapier_handle).unwrap();
                for command in commands {
                    command(rapier_body);
                }
            }
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
            let tile_shared_shape = collider_shape_cache
                .shared_shape(ColliderShape::Rectangle {
                    size: layer.tile_size,
                })
                .clone();
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

                    // Get dynamic collider if we have one
                    let dynamic_collider = self.tile_dynamic_colliders.get(tile_ent);

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

                            // Set SolverGroup based on collision kind so dynamic bodies
                            // know if they should generate contact forces with tile or not.
                            let mut simulation_membership = SolverGroup::NONE;
                            if let Some(collision_kind) = self.tile_collision_kinds.get(tile_ent) {
                                simulation_membership =
                                    collision_kind.simulation_group_membership();
                            }
                            let simulation_filter = SolverGroup::ALL;

                            // Sim group for default tile collider. This is not used for collision
                            // (only used for events) if an additional "dynamic" collider is present
                            // to be used for collision response.
                            let mut default_collider_sim_membership = simulation_membership;
                            if dynamic_collider.is_some() {
                                default_collider_sim_membership = SolverGroup::NONE;
                            }

                            // Insert default collider
                            collider_set.insert_with_parent(
                                rapier::ColliderBuilder::new(tile_shared_shape.clone())
                                    .active_events(rapier::ActiveEvents::COLLISION_EVENTS)
                                    .active_collision_types(rapier::ActiveCollisionTypes::all())
                                    .solver_groups(InteractionGroups::new(
                                        default_collider_sim_membership.bits().into(),
                                        simulation_filter.bits().into(),
                                    ))
                                    .user_data(RapierUserData::from(tile_ent)),
                                body_handle,
                                rigid_body_set,
                            );

                            // Insert dynamic collider if we have one
                            if let Some(dynamic_collider) = dynamic_collider {
                                let shared_shape =
                                    collider_shape_cache.shared_shape(dynamic_collider.shape);
                                collider_set.insert_with_parent(
                                    rapier::ColliderBuilder::new(shared_shape.clone())
                                        // Don't generate events for this collider
                                        .active_events(rapier::ActiveEvents::empty())
                                        // Only needs to collide with dynamics
                                        .active_collision_types(
                                            rapier::ActiveCollisionTypes::DYNAMIC_FIXED,
                                        )
                                        .solver_groups(InteractionGroups::new(
                                            simulation_membership.bits().into(),
                                            simulation_filter.bits().into(),
                                        ))
                                        .position(dynamic_collider.offset.into())
                                        .user_data(RapierUserData::from(tile_ent)),
                                    body_handle,
                                    rigid_body_set,
                                );
                            }
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
