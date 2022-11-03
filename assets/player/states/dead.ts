const scriptId = Script.getInfo().path;

/** Responsible for transitioning players to the dead state whenever they are killed */
export default {
  playerStateTransition() {
    const players = world.query(PlayerState);

    // Transition all players tht have been killed to this state
    for (const event of Player.killEvents()) {
      const [playerState] = players.get(event.player);
      playerState.id = Assets.getAbsolutePath("./dead.ts");
    }
  },
  handlePlayerState() {
    const players = world.query(PlayerState, AnimationBankSprite);

    for (const {
      entity,
      components: [playerState, animationBankSprite],
    } of players) {
      // In this state
      if (playerState.id != scriptId) continue;

      // Set animation when we enter the state
      if (playerState.age == 0) {
        animationBankSprite.current_animation = "death_1";
      }

      // Despawn player after 1.5 seconds ( 90 frames )
      if (playerState.age >= 90) {
        Player.despawn(entity);
        Script.setEntityState(entity, undefined);
      }
    }
  },
};
