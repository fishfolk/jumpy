type HandlePlayerMeta = HandleImage;
type PlayerControl = {
  move_direction: Vec2;
  jump_pressed: boolean;
  jump_just_pressed: boolean;
  shoot_pressed: boolean;
  shoot_just_pressed: boolean;
  grab_pressed: boolean;
  grab_just_pressed: boolean;
  slide_pressed: boolean;
  slide_just_pressed: boolean;
};
type PlayerInput = {
  active: boolean;
  selected_player: HandlePlayerMeta;
  control: PlayerControl;
  previous_control: PlayerControl;
};
type PlayerInputs = {
  players: PlayerInput[];
};
const PlayerInputs: BevyType<PlayerInputs> = {
  typeName: "jumpy::player::input::PlayerInputs",
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

type PlayerIdx = [usize];
const PlayerIdx: BevyType<PlayerIdx> = {
  typeName: "jumpy::player::PlayerIdx",
};

type AnimatedSprite = {
  start: usize;
  end: usize;
  atlas: HandleTextureAtlas;
  flip_x: boolean;
  flip_y: boolean;
  repeat: boolean;
  fps: f32;
};
const AnimatedSprite: BevyType<AnimatedSprite> = {
  typeName: "jumpy::animation::AnimatedSprite",
};
const MapMeta: BevyType<unknown> = {
  typeName: "jumpy::metadata::map::MapMeta",
};

const initState: { spawners: JsEntity[]; currentSpawner: number } = {
  currentSpawner: 0,
  spawners: [],
};

const state = ScriptInfo.state(initState);

export default {
  preUpdateInGame() {
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
        const [
          spawnerTransform,
          global_transform,
          visibility,
          computed_visibility,
        ] = world
          .query(Transform, GlobalTransform, Visibility, ComputedVisibility)
          .get(spawner);

        // Spawn the player
        const player = NetCommands.spawn();
        NetCommands.insert(player, Value.create(PlayerIdx, [i]));
        NetCommands.insert(player, spawnerTransform);
        NetCommands.insert(player, global_transform);
        NetCommands.insert(player, visibility);
        NetCommands.insert(player, computed_visibility);
        NetCommands.insert(
          player,
          Value.create(KinematicBody, {
            size: {
              x: 38,
              y: 48,
            },
            gravity: 1,
            has_friction: true,
            has_mass: true,
          })
        );
      }
    }
  },
};
