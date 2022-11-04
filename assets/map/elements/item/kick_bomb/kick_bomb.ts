const scriptPath = Script.getInfo().path;

type ItemState = null;
const itemStateInit: ItemState = null;

const COOLDOWN_FRAMES = 15;

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
            end: 6,
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
    const players = world.query(AnimatedSprite, Transform, PlayerIdx);
    const parents = world.query(Parent);
    const items = world.query(
      Item,
      Transform,
      KinematicBody,
      AnimatedSprite,
      GlobalTransform
    );

    // Update items that are being held
    //
    // This section will make the item follow the player around and match the player's facing
    // direction.
    for (const { entity: itemEnt, components } of items) {
      const [item, itemTransform, body, sprite] = components;
      if (item.script != scriptPath) continue;

      const state = Script.getEntityState<ItemState>(itemEnt, itemStateInit);

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
      // Get the current item state
      const state = Script.getEntityState<ItemState>(event.item, itemStateInit);
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
