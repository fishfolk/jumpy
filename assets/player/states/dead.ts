const scriptId = Script.getInfo().path;

const DYING_PLAYERS = "dyingPlayers";

/** Responsible for transitioning players to the dead state whenever they are killed */
export default {
  playerStateTransition() {
    const players = world.query(PlayerState, PlayerKilled);

    // Transition all players that have been killed to this state
    for (const { entity } of players) {
      if (!Script.entityListContains(DYING_PLAYERS, entity)) {
        const [playerState] = players.get(entity);
        playerState.id = Assets.getAbsolutePath("./dead.ts");
        Script.addEntityToList(DYING_PLAYERS, entity);
      }
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
        Script.removeEntityFromList(DYING_PLAYERS, entity);
        Player.despawn(entity);
        Script.setEntityState(entity, undefined);
      }
    }
  },
};
