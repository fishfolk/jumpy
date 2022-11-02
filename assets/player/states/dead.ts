const scriptId = Script.getInfo().path;

const initState = {
  frames: 0,
};

/** Responsible for transitioning players to the dead state whenever they are killed */
export default {
  playerStateTransition() {
    const players = world.query(PlayerState);

    for (const event of Player.killEvents()) {
      const [playerState] = players.get(event.player);
      playerState.id = Assets.getAbsolutePath("./dead.ts");
    }
  },
  handlePlayerState() {
    const players = world.query(PlayerState, AnimatedSprite, AnimationBankSprite);

    for (const {
      entity,
      components: [playerState, animatedSprite, animationBankSprite],
    } of players) {
      // In this state
      if (playerState.id != scriptId) continue;

      const state = Script.getEntityState(entity, initState);

      if (state.frames == 0) {
        animatedSprite.index = 0;
        animationBankSprite.current_animation = "death_1";
      }

      state.frames += 1;

      if (state.frames >= 90) {
        Player.despawn(entity);
        Script.setEntityState(entity, undefined);
      }
    }
  },
};
