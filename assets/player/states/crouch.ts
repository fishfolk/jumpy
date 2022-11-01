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

      if (!body.is_on_ground || !(control.move_direction.y < -0.5)) {
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
      KinematicBody,
      LocalPlayer
    )) {
      const [playerState, playerIdx, animationBankSprite, body] = components;

      // In this state
      if (playerState.id != scriptId) continue;

      // Set the current animation
      if (playerState.age == 0) {
        animationBankSprite.current_animation = "crouch";
      }

      // Add basic physics controls
      const control = player_inputs.players[playerIdx[0]].control;

      if (control.jump_just_pressed) {
        body.fall_through = true;
      }
    }
  },
};
