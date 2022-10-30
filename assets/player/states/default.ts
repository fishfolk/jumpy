/** Responsible for transitioning players in the default, meaningless state to "idle" */
export default {
  playerStateTransition() {
    for (const [playerState] of world
      .query(PlayerState)
      .map((x) => x.components)) {
      // Loop over players in the default state
      if (playerState.id !== "") continue;

      // Transition to the idle state
      playerState.id = Assets.getAbsolutePath("./idle.ts");
    }
  },
};
