use macroquad::{
    experimental::{
        scene,
        collections::storage,
        scene::{Node, RefMut}
    },
    color,
    rand::gen_range,
    math::Vec2,
    time::get_frame_time,
    prelude::*,
};

use crate::Resources;

pub struct FlyingGalleon {
    pub sprite: Texture2D,
    pub pos: Vec2,
    pub speed: Vec2,
    pub lived: f32,
    pub facing: bool,
    pub owner_id: u8,
}

impl FlyingGalleon {
    pub const SPEED: f32 = 200.;
    pub const LIFETIME: f32 = 15.;
    pub const WIDTH: f32 = 425.;
    pub const HEIGHT: f32 = 390.;

    pub fn new(owner_id: u8) -> FlyingGalleon {
        let resources = storage::get::<Resources>();
        let sprite = resources.items_textures["galleon/flying_galleon"];

        let (pos, facing) = Self::start_position();
        let dir = if facing {Vec2::new(1., 0.)} else { Vec2::new(-1., 0.)};

        FlyingGalleon {
            sprite,
            pos,
            speed: dir * Self::SPEED,
            lived: 0.,
            facing,
            owner_id,
        }
    }

    pub fn start_position() -> (Vec2, bool) {
        let resources = storage::get::<Resources>();
        let map_width =
            resources.tiled_map.raw_tiled_map.tilewidth * resources.tiled_map.raw_tiled_map.width;
        let map_height =
            resources.tiled_map.raw_tiled_map.tileheight * resources.tiled_map.raw_tiled_map.height;

        let facing = gen_range(0, 2) == 0;
        let pos = Vec2::new(if facing {-Self::WIDTH} else {map_width as f32}, gen_range(0., map_height as f32 - Self::HEIGHT));

        (pos, facing)
    }

    pub fn update(&mut self) -> bool {
        self.pos += self.speed * get_frame_time();
        self.lived += get_frame_time();

        if self.lived > Self::LIFETIME {
            return false;
        }

        for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
            if player.get_hitbox().overlaps(&Rect::new(self.pos.x, self.pos.y, Self::WIDTH, Self::HEIGHT)) && player.id != self.owner_id && !player.dead {
                
                scene::find_node_by_type::<crate::nodes::Camera>()
                    .unwrap()
                    .shake_noise(1.0, 10, 1.);

                {
                    let mut resources = storage::get_mut::<Resources>();
                    resources.hit_fxses.spawn(player.body.pos)
                }
                {
                    let direction = self.pos.x > (player.body.pos.x + 10.);
                    player.kill(direction);
                }
            }
        }

        true
    }

    pub fn draw(&self, pos: Vec2, facing: bool) {
        draw_texture_ex(
            self.sprite,
            pos.x,
            pos.y,
            color::WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(Self::WIDTH, Self::HEIGHT)),
                flip_x: facing,
                ..Default::default()
            },
        );
    }
}

impl Node for FlyingGalleon {
    fn draw(node: RefMut<Self>) {
        node.draw(node.pos, node.facing);
    }

    fn fixed_update(mut node: RefMut<Self>) {
        if !node.update() {
            node.delete();
        }
    }
}
