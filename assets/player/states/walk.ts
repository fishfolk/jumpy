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
      } else if (body.is_on_ground && control.move_direction.y < -0.5) {
        playerState.id = Assets.getAbsolutePath("./crouch.ts");
      } else if (control.move_direction.x == 0) {
        playerState.id = Assets.getAbsolutePath("./idle.ts");
      }
    }
  },
  handlePlayerState() {
    const player_inputs = world.resource(PlayerInputs);

    // For every player
    for (const { entity, components } of world.query(
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
        animationBankSprite.current_animation = "walk";
      }

      // Add basic physics controls
      const control = player_inputs.players[playerIdx[0]].control;

      // Add jump
      if (control.jump_just_pressed) {
        body.velocity.y = 15;
      }
      body.velocity.x = control.move_direction.x * 5;
      if (control.move_direction.x > 0) {
        animationBankSprite.flip_x = false;
      } else if (control.move_direction.x < 0) {
        animationBankSprite.flip_x = true;
      }
    }
  },
};
