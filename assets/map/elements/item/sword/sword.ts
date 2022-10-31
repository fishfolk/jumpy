const scriptPath = ScriptInfo.get().path;

export default {
  preUpdateInGame() {
    // Hydrate newly spawned sword items
    const names = world.query(EntityName);
    const items = world.query(Item);
    for (const { entity, components } of items) {
      const [item] = components;

      // If this is one of our items without a name
      if (item.script == scriptPath && !names.get(entity)) {
        info("Hydrating sword!");

        // Hydrate the entity
        world.insert(entity, Value.create(EntityName, ["Item: Sword"]));

        // Add the animated sprite
        world.insert(
          entity,
          Value.create(AnimatedSprite, {
            start: 0,
            end: 0,
            repeat: false,
            fps: 0,
            atlas: {
              id: Assets.getHandleId("sword.atlas.yaml"),
            },
          })
        );
        // And the kinematic body
        world.insert(
          entity,
          Value.create(KinematicBody, {
            size: {
              x: 64,
              y: 16,
            },
            offset: {
              y: 38,
            },
            gravity: 1,
            has_friction: true,
            has_mass: true,
          })
        );
      }
    }
  },

  updateInGame() {},
};
