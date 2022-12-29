//! Serializable data types for network messages used by the game.

use numquant::{IntRange, Quantized};

use crate::prelude::*;

pub mod match_setup;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ReliableGameMessageKind {
    MatchSetup(match_setup::MatchSetupMessage),
}

impl From<match_setup::MatchSetupMessage> for ReliableGameMessageKind {
    fn from(x: match_setup::MatchSetupMessage) -> Self {
        Self::MatchSetup(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecvReliableGameMessage {
    pub from_player_idx: usize,
    pub kind: ReliableGameMessageKind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UnreliableGameMessageKind {
    Ggrs(ggrs::Message),
}

impl From<ggrs::Message> for UnreliableGameMessageKind {
    fn from(m: ggrs::Message) -> Self {
        Self::Ggrs(m)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecvUnreliableGameMessage {
    pub from_player_idx: usize,
    pub kind: UnreliableGameMessageKind,
}

/// A resource indicating which player this game client represents, and how many players there are
/// in the match.j
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientMatchInfo {
    pub player_idx: usize,
    pub player_count: usize,
}

bitfield::bitfield! {
    /// A player's controller inputs densely packed into a single u16.
    ///
    /// This is used when sending player inputs across the network.
    #[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, PartialEq, Eq, Reflect)]
    #[repr(transparent)]
    pub struct DensePlayerControl(u32);
    impl Debug;
    jump_pressed, set_jump_pressed: 0;
    shoot_pressed, set_shoot_pressed: 1;
    grab_pressed, set_grab_pressed: 2;
    slide_pressed, set_slide_pressed: 3;
    from into DenseMoveDirection, move_direction, set_move_direction: 15, 4;
    /// This bit will be set if this player wants to try and pause or un-pause the game
    wants_to_set_pause, set_wants_to_set_pause: 16;
    /// This value is only relevant if `wants_to_set_pause` is true, and it indicates whether the
    /// player wants to pause or unpause the game.
    pause_value, set_pause_value: 17;
}

impl Default for DensePlayerControl {
    fn default() -> Self {
        let mut control = Self(0);
        control.set_move_direction(default());
        control
    }
}

/// A newtype around [`Vec2`] that implements [`From<u16>`] and [`Into<u16>`] as a way to compress
/// user stick input for use in [`DensePlayerControl`].
#[derive(Debug, Deref, DerefMut, Default)]
struct DenseMoveDirection(pub Vec2);

/// This is the specific [`Quantized`] type that we use to represent movement directions in
/// [`DenseMoveDirection`].
type MoveDirQuant = Quantized<IntRange<u32, 0b111111, -1, 1>>;

impl From<u32> for DenseMoveDirection {
    fn from(bits: u32) -> Self {
        // maximum movement value representable, we use 6 bits to represent each movement direction.
        let max = 0b111111;
        // The first six bits represent the x movement
        let x_move_bits = bits & max;
        // The second six bits represents the y movement
        let y_move_bits = (bits >> 6) & max;

        // Round near-zero values to zero
        let mut x = MoveDirQuant::from_raw(x_move_bits).to_f32();
        if x.abs() < 0.02 {
            x = 0.0;
        }
        let mut y = MoveDirQuant::from_raw(y_move_bits).to_f32();
        if y.abs() < 0.02 {
            y = 0.0;
        }

        DenseMoveDirection(Vec2::new(x, y))
    }
}

impl From<DenseMoveDirection> for u32 {
    fn from(dir: DenseMoveDirection) -> Self {
        let x_bits = MoveDirQuant::from_f32(dir.x).raw();
        let y_bits = MoveDirQuant::from_f32(dir.y).raw();

        x_bits | (y_bits << 6)
    }
}
