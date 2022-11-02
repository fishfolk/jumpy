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

  updateInGame() {
    const players = world.query(AnimatedSprite, Transform, PlayerIdx);
    const parents = world.query(Parent);
    const items = world.query(
      Transform,
      KinematicBody,
      AnimatedSprite,
      GlobalTransform,
      Item
    );

    // Update items that are being held
    for (const { entity: itemEnt, components } of items) {
      const [itemTransform, body, sprite] = components;

      let parentComponents = parents.get(itemEnt);
      if (!parentComponents) continue;
      const [parent] = parentComponents;
      const [playerSprite] = players.get(parent[0]);

      body.is_deactivated = true;
      sprite.fps = 10;
      if ((sprite.start == 8 && sprite.index == 3) || sprite.start == 0) {
        sprite.start = 4;
        sprite.end = 4;
        sprite.index = 0;
        sprite.repeat = false;
      }
      const flip = playerSprite.flip_x;
      sprite.flip_x = flip;

      itemTransform.translation = Value.create(Vec3, {
        x: 13 * (flip ? -1 : 1),
        y: 21,
      });
    }

    // Trigger used items
    for (const event of Items.useEvents()) {
      const [_itemTransform, _body, sprite] = items.get(event.item);

      let parentComponents = parents.get(event.item);
      if (!parentComponents) continue;
      const [parent] = parentComponents;
      const [playerSprite, playerTransform] = players.get(parent[0]);
      const flip = playerSprite.flip_x;

      if (sprite.start == 4) {
        sprite.index = 0;
        sprite.start = 8;
        sprite.end = 12;
        sprite.repeat = true;

        // Spawn damage region
        let entity = world.spawn();
        world.insert(
          entity,
          Value.create(Transform, {
            translation: {
              x: playerTransform.translation.x + 20 * (flip ? -1 : 1),
              y: playerTransform.translation.y + 20,
            },
          })
        );
        world.insert(
          entity,
          Value.create(DamageRegion, {
            size: {
              x: 50,
              y: 80,
            },
          })
        );
        world.insert(entity, Value.create(DamageRegionOwner, [parent[0]]));
        world.insert(
          entity,
          Value.create(Lifetime, {
            lifetime: 0.1,
          })
        );
      }
    }

    // Update dropped items
    for (const event of Items.dropEvents()) {
      const [item_transform, body, sprite] = items.get(event.item);

      body.is_deactivated = false;
      sprite.start = 0;
      sprite.end = 0;
      body.velocity = event.velocity;
      body.is_spawning = true;

      item_transform.translation.y = event.position.y - 30;
      item_transform.translation.x = event.position.x;
      item_transform.translation.z = event.position.z;
    }
  },
};
