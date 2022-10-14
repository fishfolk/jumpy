type PlayerState = [number];
const PlayerState: BevyType<PlayerState> = {
  typeName: "jumpy::player::state::PlayerState",
};

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

const scriptId = ScriptInfo.get().handle_id_hash;

export default {
  handlePlayerState() {
    const player_inputs = world.resource(PlayerInputs);

    // For every player
    for (const [playerState, playerIdx, body] of world
      .query(PlayerState, PlayerIdx, KinematicBody)
      .map((x) => x.components)) {

      // In this state
      if (playerState[0] != scriptId) continue;

      // Add basic physics controls
      const control = player_inputs.players[playerIdx[0]].control;
      if (body.is_on_ground && control.shoot_just_pressed) {
        body.velocity.y = 14.0;
      }
      body.velocity.x = control.move_direction.x * 5;
    }
  },
};
