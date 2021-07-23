use macroquad::{
    experimental::animation::{AnimatedSprite, Animation},
    prelude::{scene::RefMut, *},
};

const FLAPPY_JELLYFISH_WIDTH: f32 = 50.;
pub const FLAPPY_JELLYFISH_HEIGHT: f32 = 51.;
const FLAPPY_JELLYFISH_ANIMATION_FLAPPY: &'static str = "flappy";

/// The FlappyJellyfish doesn't have a body, as it has a non-conventional (flappy bird-style) motion.
pub struct FlappyJellyfish {
    flappy_jellyfish_sprite: AnimatedSprite,
    current_pos: Vec2,
    owner_id: u8,
}

impl FlappyJellyfish {
    /// Should not be called; spawn() should called instead, which handles the node.
    fn new(jellyfish_pos: Vec2, owner_id: u8) -> Self {
        let flappy_jellyfish_sprite = AnimatedSprite::new(
            FLAPPY_JELLYFISH_WIDTH as u32,
            FLAPPY_JELLYFISH_HEIGHT as u32,
            &[Animation {
                name: FLAPPY_JELLYFISH_ANIMATION_FLAPPY.to_string(),
                row: 0,
                frames: 8,
                fps: 8,
            }],
            true,
        );

        Self {
            flappy_jellyfish_sprite,
            current_pos: jellyfish_pos,
            owner_id,
        }
    }

    pub fn spawn(jellyfish_pos: Vec2, owner_id: u8) {
        let flappy_jellyfish = Self::new(jellyfish_pos, owner_id);

        scene::add_node(flappy_jellyfish);
    }
}

impl scene::Node for FlappyJellyfish {
    fn fixed_update(mut node: RefMut<Self>) {
        todo!("WRITEME: FlappyJellyfish#fixed_update");
    }

    fn draw(mut node: RefMut<Self>) {
        todo!("WRITEME: FlappyJellyfish#draw");
    }
}
