/**
 * TODO: Migrate to rust.
 * 
 * This script isn't currently running.
 */
const killZoneBorder = 500;

export default {
  updateInGame() {
    let netInfo = NetInfo.get();
    let map = world.query(MapMeta).map((x) => x.components[0])[0];
    if (!map) return;

    let mapWidth = map.grid_size.x * map.tile_size.x;
    let leftKillZone = -killZoneBorder;
    let rightKillZone = killZoneBorder + mapWidth;
    let bottomKillZone = -killZoneBorder;

    for (const { entity, components } of world.query(PlayerIdx, Transform)) {
      let [player_idx, transform] = components;

      let pos = transform.translation;

      if (
        pos.x < leftKillZone ||
        pos.x > rightKillZone ||
        pos.y < bottomKillZone
      ) {
        Player.kill(entity);
      }
    }
  },
};
