const initState: { spawners: JsEntity[]; currentSpawner: number } = {
  currentSpawner: 0,
  spawners: [],
};

const state = Script.state(initState);

export default {
  preUpdate() {
    if (NetInfo.get().is_client) return;

    const player_inputs = world.resource(PlayerInputs);

    const mapQuery = world.query(MapMeta)[0];
    if (!mapQuery) {
      state.spawners = [];
      return;
    }

    const spawnedEntities = MapElement.getSpawnedEntities();
    if (spawnedEntities.length > 0) {
      state.spawners = spawnedEntities.map((e) => EntityRef.toJs(e));
    }

    // Collect all the alive players on the map
    const alive_players = world.query(PlayerIdx).map((x) => x.components[0][0]);

    // For every player
    for (let i = 0; i < 4; i++) {
      // Get the player input
      const player = player_inputs.players[i];

      // If the player is active, but not alive
      if (player.active && !alive_players.includes(i)) {
        // Get the next spawner
        state.currentSpawner += 1;
        state.currentSpawner %= state.spawners.length;

        const spawner = EntityRef.fromJs(state.spawners[state.currentSpawner]);

        // Get the spawner transform
        const [spawnerTransform] = world.query(Transform).get(spawner);

        // Spawn the player
        const player = world.spawn();
        world.insert(player, Value.create(PlayerIdx, [i]));
        world.insert(player, spawnerTransform);
      }
    }
  },
};
