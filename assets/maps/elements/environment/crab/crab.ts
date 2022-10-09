type AnimatedSprite = {
  start: usize;
  end: usize;
  atlas: HandleTextureAtlas;
  flip_x: boolean;
  flip_y: boolean;
  repeat: boolean;
  fps: f32;
};
const AnimatedSprite: BevyType<AnimatedSprite> = {
  typeName: "jumpy::animation::AnimatedSprite",
};

type KinematicBody = {
  offset: Vec2;
  size: Vec2;
  velocity: Vec2;
  is_on_ground: boolean;
  was_on_ground: boolean;
  has_mass: boolean;
  has_friction: boolean;
  bouncyness: f32;
  is_deactivated: boolean;
  gravity: f32;
};
const KinematicBody: BevyType<KinematicBody> = {
  typeName: "jumpy::physics::KinematicBody",
};

type EntityName = [string];
const EntityName: BevyType<EntityName> = {
  typeName: "jumpy::name::EntityName",
};

const initState: { crabs: Entity[] } = {
  crabs: [],
};

export default {
  preUpdate() {
    const state = ScriptInfo.state(initState);

    const spawnedEntities = MapElement.getSpawnedEntities();
    if (spawnedEntities.length > 0) {
      state.crabs = [];
    }

    // Handle newly spawned map entities
    for (const spanwer_entity of spawnedEntities) {
      const [transform, global_transform, computed_visibility] = world
        .query(Transform, GlobalTransform, ComputedVisibility)
        .get(spanwer_entity);

      // Spawn a new entity for the crab and copy the transform and visibility from the map element
      const entity = world.spawn();
      state.crabs.push(entity);

      world.insert(entity, Value.create(EntityName, ["Critter: Crab"]));
      world.insert(entity, transform);
      world.insert(entity, global_transform);
      world.insert(entity, computed_visibility);
      world.insert(entity, Value.create(Visibility));

      // Add the animated sprite
      world.insert(
        entity,
        Value.create(AnimatedSprite, {
          start: 0,
          end: 1,
          repeat: true,
          fps: 3,
          atlas: {
            id: Assets.getHandleId("crab.atlas.yaml"),
          },
        })
      );

      // And the kinematic body
      world.insert(
        entity,
        Value.create(KinematicBody, {
          size: {
            x: 17,
            y: 12,
          },
          velocity: {
            x: 5,
            y: 10,
          },
          gravity: 1,
          bouncyness: 0.5,
          has_friction: true,
          has_mass: true,
        })
      );
    }
  },

  update() {
    const state = ScriptInfo.state(initState);
    const query = world.query(KinematicBody, Transform);

    for (const crab of state.crabs) {
      const [kinematicBody, transform] = query.get(crab);
    }
  },
};
