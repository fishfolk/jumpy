type PlayerState = { id: number; age: number };
const PlayerState: BevyType<PlayerState> = {
  typeName: "jumpy::player::state::PlayerState",
};

/** Responsible for transitioning players in the default, meaningless state, to the default state,
 * "idle" */
export default {
  playerStateTransition() {
    for (const [playerState] of world
      .query(PlayerState)
      .map((x) => x.components)) {
      // Loop over players in the default state
      if (playerState.id !== 0) continue;

      // Transition to the idle state
      playerState.id = Assets.getHandleId("./idle.ts").hash();
    }
  },
};
