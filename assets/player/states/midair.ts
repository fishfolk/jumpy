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

    // For every player
    const playerComponents = world
      .query(PlayerState, PlayerIdx, AnimationBankSprite, KinematicBody)
      .map((x) => x.components);

    for (const [
      playerState,
      playerIdx,
      animationBankSprite,
      body,
    ] of playerComponents) {
      if (playerState.id != scriptId) continue;

      // Set the current animation
      if (body.velocity.y > 0) {
        animationBankSprite.current_animation = "rise";
      } else {
        animationBankSprite.current_animation = "fall";
      }

      // Add controls
      const control = player_inputs.players[playerIdx[0]].control;
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
