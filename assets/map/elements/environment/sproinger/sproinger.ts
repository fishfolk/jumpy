// Define the sproinger state types
type SproingerState = {
  sproinging: boolean;
  frame: number;
};

// Add our constants
const FORCE = 25;

export default {
  preUpdateInGame() {
    // Check for the existence of the map
    const map = world.query(MapMeta)[0];
    // If there is no map
    if (!map) {
      // clear our sproinger list
      Script.clearEntityList("sproingers");
      return;
    }

    // Get the list of spawned entities
    const spawnedEntities = MapElement.getSpawnedEntities();
    // If there are spawned entities, that means the map was loaded or reloaded.
    if (spawnedEntities.length > 0) {
      // So clear our sproinger list
      Script.clearEntityList("sproingers");
    }

    // For every new sproinger entity
    for (const entity of spawnedEntities) {
      // Add this entity to our list of sproingers.
      //
      // Note: Because we cannot persist entity refs across frames,
      // we must first convert the entity to a JSON representation.
      Script.addEntityToList("sproingers", entity);

      // Add the sprite
      world.insert(
        entity,
        Value.create(AnimatedSprite, {
          start: 0,
          end: 6,
          repeat: false,
          fps: 0,
          atlas: {
            id: Assets.getHandleId("./sproinger.atlas.yaml"),
          },
        })
      );
      // And the physics body
      world.insert(
        entity,
        Value.create(KinematicBody, {
          size: {
            x: 32,
            y: 8,
          },
          offset: {
            y: -6,
          },
          has_mass: false,
        })
      );
    }
  },

  updateInGame() {
    const bodies = world.query(KinematicBody);
    const animatedSprites = world.query(AnimatedSprite);

    // Loop over all our sproingers
    for (const entity of Script.getEntityList("sproingers")) {
      // Get our sproinger sprite
      const [sprite] = animatedSprites.get(entity);

      // Get the script-local state for the sproinger entity
      const entState = Script.getEntityState<SproingerState>(entity, {
        frame: 0,
        sproinging: false,
      });

      // If the sproinger is currently sproinging
      if (entState.sproinging) {
        // Play the sproinging animation
        switch (entState.frame) {
          case 0:
            sprite.index = 2;
            break;
          case 4:
            sprite.index = 3;
            break;
          case 8:
            sprite.index = 4;
            break;
          case 12:
            sprite.index = 5;
            break;
          case 20:
            sprite.index = 0;
            entState.sproinging = false;
            entState.frame = 0;
            break;
        }
        entState.frame += 1;
      }

      // See if the spoinger has any collisions
      for (const collidedEntity of CollisionWorld.actorCollisions(entity)) {
        // Get the kinematic body of the collided entity
        const components = bodies.get(collidedEntity);
        if (!components) continue;
        const [body] = components;

        if (!entState.sproinging) {
          // Apply the sproing force to the body
          body.velocity.y = FORCE;

          // Go into a sproinging state
          entState.sproinging = true;
        }
      }
    }
  },
};
