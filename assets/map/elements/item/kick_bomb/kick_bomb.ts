const scriptPath = Script.getInfo().path;

type LitBombState = {
  frames: number;
};
type ScriptState = {
  litBombs: JsEntity[];
};
const scriptState = Script.state<ScriptState>({
  litBombs: [],
});

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
        world.insert(entity, Value.create(EntityName, ["Item: Kick Bomb"]));

        // Add the animated sprite
        world.insert(
          entity,
          Value.create(AnimatedSprite, {
            start: 0,
            end: 5,
            repeat: false,
            fps: 0,
            atlas: {
              id: Assets.getHandleId("kick_bomb.atlas.yaml"),
            },
          })
        );
        // And the kinematic body
        world.insert(
          entity,
          Value.create(KinematicBody, {
            size: {
              x: 26,
              y: 26,
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
    const players = world.query(
      AnimatedSprite,
      Transform,
      PlayerIdx,
      GlobalTransform,
      ComputedVisibility
    );
    const parents = world.query(Parent);
    const items = world.query(
      Item,
      Transform,
      KinematicBody,
      AnimatedSprite,
      GlobalTransform
    );
    const transforms = world.query(
      Transform,
      GlobalTransform,
      Visibility,
      ComputedVisibility
    );

    // Update items that are being held
    //
    // This section will make the item follow the player around and match the player's facing
    // direction.
    for (const { entity: itemEnt, components } of items) {
      const [item, itemTransform, body, sprite] = components;
      if (item.script != scriptPath) continue;

      let parentComponents = parents.get(itemEnt);
      // If this item isn't being held, skip the item
      if (!parentComponents) continue;

      const [parent] = parentComponents;
      const [playerSprite] = players.get(parent[0]);

      // Deactivate item collision
      body.is_deactivated = true;

      // Set animation to default position if we are being held
      sprite.index = 0;

      // Flip the sprite to match our player orientation
      const flip = playerSprite.flip_x;
      sprite.flip_x = flip;
      const flipFactor = flip ? -1 : 1;
      // Align the sprite with the player's position
      itemTransform.translation = Value.create(Vec3, {
        x: 13 * flipFactor,
        y: 0,
      });
    }

    // For every item that is being used
    for (const event of Items.useEvents()) {
      let parentComponents = parents.get(event.item);
      // If this item isn't being held, skip the item
      if (!parentComponents) continue;

      // Get the player info
      const [parent] = parentComponents;
      const [
        playerSprite,
        transform,
        _idx,
        globalTransform,
        computedVisibility,
      ] = players.get(parent[0]);
      const flip = playerSprite.flip_x;
      const flipFactor = flip ? -1 : 1;

      // Despawn the item from the player's hand
      WorldTemp.despawnRecursive(event.item);

      // Spawn a new, lit bomb to the map
      const entity = WorldTemp.spawn();
      scriptState.litBombs.push(EntityRef.toJs(entity));
      world.insert(entity, transform);
      world.insert(entity, globalTransform);
      world.insert(entity, computedVisibility);
      world.insert(entity, Value.create(Visibility));

      // Add the animated sprite
      world.insert(
        entity,
        Value.create(AnimatedSprite, {
          start: 3,
          end: 5,
          repeat: true,
          fps: 8,
          atlas: {
            id: Assets.getHandleId("kick_bomb.atlas.yaml"),
          },
        })
      );
      // And the kinematic body
      world.insert(
        entity,
        Value.create(KinematicBody, {
          size: {
            x: 26,
            y: 26,
          },
          velocity: {
            x: 10 * flipFactor,
          },
          gravity: 1,
          has_friction: true,
          has_mass: true,
        })
      );
    }

    // Handle lit bombs
    const litBombs = scriptState.litBombs;
    scriptState.litBombs = [];
    for (const jsEntity of litBombs) {
      const bombEntity = EntityRef.fromJs(jsEntity);

      // Get the bomb's state
      const state = Script.getEntityState<LitBombState>(bombEntity, {
        frames: 0,
      });
      const [transform, globalTransform, visibility, computedVisibility] =
        transforms.get(bombEntity);

      if (state.frames >= 60) {
        // Spawn damage region entity
        const damageRegionEnt = WorldTemp.spawn();
        world.insert(damageRegionEnt, transform);
        world.insert(damageRegionEnt, globalTransform);
        world.insert(damageRegionEnt, visibility);
        world.insert(damageRegionEnt, computedVisibility);
        world.insert(
          damageRegionEnt,
          Value.create(DamageRegion, {
            size: {
              x: 26 * 3.5,
              y: 26 * 3.5,
            },
          })
        );
        world.insert(
          damageRegionEnt,
          Value.create(Lifetime, {
            lifetime: (1 / 9) * 4,
          })
        );
        // Spawn explosion sprite entity
        const explosionSpriteEnt = WorldTemp.spawn();
        world.insert(explosionSpriteEnt, transform);
        world.insert(explosionSpriteEnt, globalTransform);
        world.insert(explosionSpriteEnt, visibility);
        world.insert(explosionSpriteEnt, computedVisibility);
        world.insert(
          explosionSpriteEnt,
          Value.create(AnimatedSprite, {
            start: 0,
            end: 11,
            repeat: false,
            fps: 9,
            atlas: {
              id: Assets.getHandleId("explosion.atlas.yaml"),
            },
          })
        );
        world.insert(
          explosionSpriteEnt,
          Value.create(Lifetime, {
            lifetime: (1 / 9) * 11,
          })
        );

        // Despawn the lit bomb
        WorldTemp.despawnRecursive(bombEntity);
      } else {
        state.frames += 1;
        scriptState.litBombs.push(jsEntity);
      }
    }

    // Update dropped items
    for (const event of Items.dropEvents()) {
      const [_item, itemTransform, body, sprite] = items.get(event.item);
      let flip = sprite.flip_x;
      let flipFactor = flip ? -1 : 1;

      // Re-activate physics body on the item
      body.is_deactivated = false;
      // Make sure item maintains player velocity
      body.velocity = event.velocity;
      body.is_spawning = true;

      // Drop item at the middle of the player
      itemTransform.translation.y = event.position.y;
      itemTransform.translation.x = event.position.x + 13 * flipFactor;
      itemTransform.translation.z = event.position.z;
    }
  },
};
