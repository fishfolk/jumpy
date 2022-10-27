type PlayerIdx = [usize];
const PlayerIdx: BevyType<PlayerIdx> = {
  typeName: "jumpy::player::PlayerIdx",
};

const GameCamera: BevyType<unknown> = {
  typeName: "jumpy::camera::GameCamera",
};
const MapMeta: BevyType<{
  grid_size: UVec2;
  tile_size: UVec2;
}> = {
  typeName: "jumpy::metadata::map::MapMeta",
};

type KinematicBody = {
  offset: Vec2;
  size: Vec2;
  velocity: Vec2;
  is_on_ground: boolean;
  was_on_ground: boolean;
  has_mass: boolean;
  has_friction: boolean;
  bouncyness: f32;
  is_deactivated: boolean;
  gravity: f32;
};
const KinematicBody: BevyType<KinematicBody> = {
  typeName: "jumpy::physics::KinematicBody",
};

const lerpFactor = 0.1;

export default {
  postUpdateInGame() {
    const mapQuery = world.query(MapMeta)[0];
    if (!mapQuery) return;

    const playerComponents = world
      .query(PlayerIdx, Transform)
      .map((x) => x.components);

    const [_, cameraTransform, projection] = world.query(
      GameCamera,
      Transform,
      OrthographicProjection
    )[0].components;

    let middlePoint = { x: 0, y: 0 };
    let min = { x: 100000, y: 100000 };
    let max = { x: -100000, y: -100000 };

    const player_count = playerComponents.length;

    for (const [_, playerTransform] of playerComponents) {
      const playerPos = playerTransform.translation;
      middlePoint.x += playerPos.x;
      middlePoint.y += playerPos.y;

      min.x = Math.min(playerPos.x, min.x);
      min.y = Math.min(playerPos.y, min.y);
      max.x = Math.max(playerPos.x, max.x);
      max.y = Math.max(playerPos.y, max.y);
    }

    middlePoint.x /= Math.max(player_count, 1);
    middlePoint.y /= Math.max(player_count, 1);

    for (const dim of ["x", "y"]) {
      let delta = cameraTransform.translation[dim] - middlePoint[dim];
      let dist = delta * lerpFactor;
      cameraTransform.translation[dim] -= dist;
    }

    projection.scale = 1.25;
  },
};
