use hecs::World;
use macroquad::prelude::*;

use crate::network::AccountId;
use crate::{collect_local_input, GameInput, GameInputScheme};

#[derive(Debug, Copy, Clone)]
pub enum PlayerControllerKind {
    LocalInput(GameInputScheme),
    Network(AccountId),
}

impl PlayerControllerKind {
    pub fn is_local(&self) -> bool {
        if let PlayerControllerKind::LocalInput(..) = self {
            return true;
        }

        false
    }
}

pub struct PlayerController {
    pub kind: PlayerControllerKind,

    /// No vertical movement is possible now but you never know what the future holds :)
    pub move_direction: Vec2,

    pub should_crouch: bool,
    pub should_jump: bool,
    pub should_float: bool,
    pub should_pickup: bool,
    pub should_attack: bool,
    pub should_slide: bool,
}

impl From<PlayerControllerKind> for PlayerController {
    fn from(kind: PlayerControllerKind) -> Self {
        PlayerController {
            kind,
            move_direction: Vec2::ZERO,
            should_crouch: false,
            should_jump: false,
            should_float: false,
            should_pickup: false,
            should_attack: false,
            should_slide: false,
        }
    }
}

impl PlayerController {
    pub fn clear(&mut self) {
        self.move_direction = Vec2::ZERO;
        self.should_crouch = false;
        self.should_jump = false;
        self.should_float = false;
        self.should_pickup = false;
        self.should_attack = false;
        self.should_slide = false;
    }

    pub fn apply_input(&mut self, input: GameInput) {
        self.clear();

        if input.left {
            self.move_direction.x -= 1.0;
        }

        if input.right {
            self.move_direction.x += 1.0;
        }

        self.should_crouch = input.down;
        self.should_jump = input.jump;
        self.should_float = input.float;
        self.should_pickup = input.pickup;
        self.should_attack = input.fire;
        self.should_slide = input.slide;
    }
}

pub fn update_player_controllers(world: &mut World) {
    for (_, controller) in world.query_mut::<&mut PlayerController>() {
        match controller.kind {
            PlayerControllerKind::LocalInput(input_scheme) => {
                let input = collect_local_input(input_scheme);
                controller.apply_input(input);
            }
            PlayerControllerKind::Network(_account_id) => {
                // TODO: Network input
            }
        }
    }
}
