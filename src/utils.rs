use macroquad::{
    experimental::collections::storage, experimental::scene, experimental::scene::RefMut,
    prelude::Vec2, rand::gen_range,
};

use crate::{nodes::Player, Circle, Resources};
use std::f32;

/// Creates explosion FX and kills players within `radius`
pub fn explode(position: Vec2, radius: f32) {
    let mut r = 0.0;
    {
        let mut resources = storage::get_mut::<Resources>();
        while r < radius - 5. {
            // Particles are 5 in radius
            r += 10.0 / (r + 1.);
            let angle = gen_range(0.0, f32::consts::PI * 2.);
            resources
                .fx_explosion_fire
                .emit(position + Vec2::new(angle.cos(), angle.sin()) * r, 1); //Explosion
        }

        resources.fx_explosion_particles.spawn(position); //Bits/particles

        let mut a = 0.0;
        while a < f32::consts::PI * 2.0 {
            resources
                .fx_smoke
                .emit(position + Vec2::new(a.cos(), a.sin()) * (radius - 15.0), 1); //Smoke at the edges of the explosion
            a += 4.0 / radius;
        }
    }

    for mut player in player_circle_hit(position, radius) {
        let xpos = player.body.pos.x;
        let hitbox_width = player.get_hitbox().x;
        player.kill(position.x > (xpos + hitbox_width / 2.)); //Verify player is thrown in correct direction
    }

    scene::find_node_by_type::<crate::nodes::Camera>()
        .unwrap()
        .shake_noise(radius / 3., 15, 0.6);
    scene::find_node_by_type::<crate::nodes::Camera>()
        .unwrap()
        .shake_rotational(1., 5);
}

pub fn player_circle_hit(position: Vec2, radius: f32) -> Vec<RefMut<Player>> {
    let hitbox = Circle::new(position.x, position.y, radius);

    scene::find_nodes_by_type::<crate::nodes::Player>()
        .filter(|player| hitbox.overlaps_rect(&player.get_hitbox()))
        .collect::<Vec<RefMut<Player>>>()
}
