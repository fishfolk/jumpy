const scriptId = ScriptInfo.get().path;

export default {
  playerStateTransition() {
    const player_inputs = world.resource(PlayerInputs);
    const playerComponents = world
      .query(PlayerState, PlayerIdx, KinematicBody)
      .map((x) => x.components);

    for (const [playerState, playerIdx, body] of playerComponents) {
      if (playerState.id != scriptId) continue;

      const control = player_inputs.players[playerIdx[0]].control;

      if (!body.is_on_ground) {
        playerState.id = Assets.getAbsolutePath("./midair.ts");
      } else if (control.move_direction.y < -0.5) {
        playerState.id = Assets.getAbsolutePath("./crouch.ts");
      } else if (control.move_direction.x != 0) {
        playerState.id = Assets.getAbsolutePath("./walk.ts");
      }
    }
  },
  handlePlayerState() {
    const player_inputs = world.resource(PlayerInputs);

    let items = world.query(Item);

    // For every player
    for (const { entity: playerEnt, components } of world.query(
      PlayerState,
      PlayerIdx,
      AnimationBankSprite,
      KinematicBody
    )) {
      const [playerState, playerIdx, animationBankSprite, body] = components;

      // In this state
      if (playerState.id != scriptId) continue;

      // Set the current animation
      if (playerState.age == 0) {
        animationBankSprite.current_animation = "idle";
      }

      // Add basic physics controls
      const control = player_inputs.players[playerIdx[0]].control;

      // If we are grabbing
      if (control.grab_just_pressed) {
        // Get inventory
        const currentInventory = Player.getInventory(playerEnt);

        if (!currentInventory) {
          // For each actor colliding with the player
          for (const collider of CollisionWorld.actorCollisions(playerEnt)) {
            const item = items.get(collider);
            if (!!item) {
              Player.setInventory(playerEnt, collider);
              break;
            }
          }
        } else {
          Player.setInventory(playerEnt, null);
        }
      }

      // Use item if we have one
      if (control.shoot_just_pressed && Player.getInventory(playerEnt)) {
        Player.useItem(playerEnt);
      }

      if (control.jump_just_pressed) {
        body.velocity.y = 15;
      }
      body.velocity.x = 0;
    }
  },
};
