const scriptId = ScriptInfo.get().path;

export default {
  playerStateTransition() {
    const playerComponents = world
      .query(PlayerState, KinematicBody)
      .map((x) => x.components);

    for (const [playerState, body] of playerComponents) {
      if (playerState.id != scriptId) continue;

      if (body.is_on_ground) {
        playerState.id = Assets.getAbsolutePath("./idle.ts");
      }
    }
  },
  handlePlayerState() {
    const player_inputs = world.resource(PlayerInputs);
    const items = world.query(Item);

    // For every player
    for (const { entity: playerEnt, components } of world.query(
      PlayerState,
      PlayerIdx,
      AnimationBankSprite,
      KinematicBody,
      LocalPlayer
    )) {
      const [playerState, playerIdx, animationBankSprite, body] = components;
      if (playerState.id != scriptId) continue;

      // Set the current animation
      if (body.velocity.y > 0) {
        animationBankSprite.current_animation = "rise";
      } else {
        animationBankSprite.current_animation = "fall";
      }

      const control = player_inputs.players[playerIdx[0]].control;

      // If we are grabbing
      if (control.grab_just_pressed) {
        const current_inventory = Player.getInventory(playerEnt);
        if (!current_inventory) {
          // For each actor colliding with the player
          for (const collider of CollisionWorld.actorCollisions(playerEnt)) {
            const item = items.get(collider);
            if (!!item) {
              const [_item] = item;
              Player.setInventory(playerEnt, collider);
              break;
            }
          }
        } else {
          Player.setInventory(playerEnt, null);
        }
      }

      // Add controls
      body.velocity.x = control.move_direction.x * 5;

      // Fall through platforms when pressing down
      if (control.move_direction.y < -0.5 && control.jump_pressed) {
        body.fall_through = true;
      } else {
        body.fall_through = false;
      }

      if (body.velocity.x > 0) {
        animationBankSprite.flip_x = false;
      } else if (body.velocity.x < 0) {
        animationBankSprite.flip_x = true;
      }
    }
  },
};
