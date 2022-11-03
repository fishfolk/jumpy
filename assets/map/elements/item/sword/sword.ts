const scriptPath = Script.getInfo().path;

type Idle = { status: "idle" };
type Swinging = { status: "swinging"; frame: number };
type Cooldown = { status: "cooldown"; frame: number };
type ItemState = Idle | Swinging | Cooldown;
const itemStateInit: ItemState = { status: "idle" };

const COOLDOWN_FRAMES = 15;
const ATTACK_FPS = 10;

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
            fps: ATTACK_FPS,
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

    // Helper to spawn a damage region
    const spawnDamageRegion = (
      owner: Entity,
      x: number,
      y: number,
      width: number,
      height: number
    ) => {
      /// This is a hack to get a global transform because scripts can't construct it with
      /// Value.create(). ( Fixed in Bevy 0.9 )
      const globalTransform = items[0].components[3];

      // Spawn damage region entity
      let entity = world.spawn();
      world.insert(
        entity,
        Value.create(Transform, {
          translation: {
            x,
            y,
          },
        })
      );
      world.insert(entity, globalTransform);
      world.insert(
        entity,
        Value.create(DamageRegion, {
          size: {
            x: width,
            y: height,
          },
        })
      );
      world.insert(entity, Value.create(DamageRegionOwner, [owner]));
      world.insert(
        entity,
        Value.create(Lifetime, {
          lifetime: 1 / 60,
        })
      );
    };

    // Update items that are being held
    //
    // This section will make the item follow the player around and match the player's facing
    // direction.
    for (const { entity: itemEnt, components } of items) {
      const [itemTransform, body, sprite] = components;
      const state = Script.getEntityState<ItemState>(itemEnt, itemStateInit);

      let parentComponents = parents.get(itemEnt);
      // If this item isn't being held, skip the item
      if (!parentComponents) continue;

      const [parent] = parentComponents;
      const [playerSprite, playerTransform] = players.get(parent[0]);

      // Deactivate item collision
      body.is_deactivated = true;

      // Set animation to default position if we aren't swinging
      if (state.status != "swinging") {
        sprite.start = 4;
        sprite.end = 4;
        sprite.index = 0;
        sprite.repeat = false;
      }

      // Flip the sprite to match our player orientation
      const flip = playerSprite.flip_x;
      sprite.flip_x = flip;
      const flipFactor = flip ? -1 : 1;
      // Align the sprite with the player's position
      itemTransform.translation = Value.create(Vec3, {
        x: 13 * (flip ? -1 : 1),
        y: 21,
      });

      // If we're swinging the weapon
      if (state.status == "swinging") {
        // If we are at the end of the swing animation
        if (sprite.index >= sprite.end - sprite.start - 1) {
          // Go to cooldown frames
          Script.setEntityState<ItemState>(itemEnt, {
            status: "cooldown",
            frame: 0,
          });

          // Set the current attack frame to the animation frame index
        } else {
          state.frame = sprite.index;
        }

        // Trigger frame collisions for each sword animation position
        switch (state.frame) {
          case 0:
            spawnDamageRegion(
              parent[0],
              playerTransform.translation.x + 20 * flipFactor,
              playerTransform.translation.y + 20,
              30,
              70
            );
          case 1:
            spawnDamageRegion(
              parent[0],
              playerTransform.translation.x + 25 * flipFactor,
              playerTransform.translation.y + 20,
              40,
              50
            );
          case 2:
            spawnDamageRegion(
              parent[0],
              playerTransform.translation.x + 20 * flipFactor,
              playerTransform.translation.y,
              40,
              50
            );
        }

        state.frame += 1;

        // If we are in cooldown frames
      } else if (state.status == "cooldown") {
        // If cooldown frames have finished
        if (state.frame >= COOLDOWN_FRAMES) {
          // Go back to idle state
          Script.setEntityState<ItemState>(itemEnt, { status: "idle" });
        } else {
          state.frame += 1;
        }
      }
    }

    // For every item that is being used
    for (const event of Items.useEvents()) {
      // Get the current item state
      const state = Script.getEntityState<ItemState>(event.item, itemStateInit);

      if (state.status == "idle") {
        const [_itemTransform, _body, sprite] = items.get(event.item);

        // Start attacking animation
        sprite.index = 0;
        sprite.start = 8;
        sprite.end = 12;

        // And move to an attacking state
        Script.setEntityState<ItemState>(event.item, {
          status: "swinging",
          frame: 0,
        });
      }
    }

    // Update dropped items
    for (const event of Items.dropEvents()) {
      const [item_transform, body, sprite] = items.get(event.item);

      // Re-activate physics body on the item
      body.is_deactivated = false;
      // Put sword in rest position
      sprite.start = 0;
      sprite.end = 0;
      // Make sure item maintains player velocity
      body.velocity = event.velocity;
      body.is_spawning = true;

      // Drop item at the middle of the player
      item_transform.translation.y = event.position.y - 30;
      item_transform.translation.x = event.position.x;
      item_transform.translation.z = event.position.z;
    }
  },
};
