use macroquad::{
    color,
    experimental::{animation::AnimatedSprite, collections::storage, scene::RefMut},
    prelude::*,
    rand::gen_range,
};

use crate::{components::PhysicsBody, nodes::Player, Resources, GameWorld};

use std::f32;

pub struct Explosive {
    texture: String,
    sprite: AnimatedSprite,
    body: PhysicsBody,
    lived: f32,
    owner_id: u8,
    detonation_parameters: DetonationParameters,
}

pub struct DetonationParameters {
    /// If a player is within this radius, it will explode. `None` for no proximity detonation.
    pub trigger_radius: Option<f32>,
    /// Duration for which the owner of the explosive will not trigger the proximity fuze.
    pub owner_safe_fuse: f32,
    /// Radius of the explosion
    pub explosion_radius: f32,
    /// Time until explosive explodes. `None` for no automatic detonation
    pub fuse: Option<f32>,
}

impl Explosive {
    // Use Explosive::spawn(), which handles the scene graph.
    //
    fn new(
        pos: Vec2,
        velocity: Vec2,
        detonation_parameters: DetonationParameters,
        texture: &str,
        sprite: AnimatedSprite,
        size: Vec2,
        owner_id: u8,
    ) -> Self {
        // This can be easily turned into a single sprite, rotated via DrawTextureParams.
        //
        let mut world = storage::get_mut::<GameWorld>();

        let mut body = PhysicsBody::new(&mut world.collision_world, pos, 0.0, size);
        body.speed = velocity;

        Self {
            texture: texture.to_string(),
            sprite,
            body,
            lived: 0.0,
            owner_id,
            detonation_parameters,
        }
    }
    /// Creates a new explosive and adds it to the scene.
    /// # Arguments
    /// * `pos` - Position of the explosive
    /// * `velocity` - Velocity of the explosive
    /// * `detonation_parameters` - Configuration for how and when it explodes
    /// * `texture` - Name of the texture (eg. "cannon/cannonball")
    /// * `sprite` - An animated sprites for an animated explosive
    /// * `size` - Dimensions of the explosive, for physics and rendering
    /// * `owned_id` - ID of the player who created the explosive, for preventing blowing up the owner.
    pub fn spawn(
        pos: Vec2,
        velocity: Vec2,
        detonation_parameters: DetonationParameters,
        texture: &str,
        sprite: AnimatedSprite,
        size: Vec2,
        owner_id: u8,
    ) {
        let explosive = Explosive::new(
            pos,
            velocity,
            detonation_parameters,
            texture,
            sprite,
            size,
            owner_id,
        );
        scene::add_node(explosive);
    }
}

impl scene::Node for Explosive {
    fn fixed_update(mut explosive: RefMut<Self>) {
        explosive.body.update();
        explosive.lived += get_frame_time();

        let explosion_position = explosive.body.pos + explosive.body.size / 2.;

        let mut explode = false;

        if let Some(fuse) = explosive.detonation_parameters.fuse {
            if explosive.lived > fuse {
                explode = true;
            }
        }

        if let Some(radius) = explosive.detonation_parameters.trigger_radius {
            for player in player_circle_hit(explosion_position, radius) {
                if explosive.lived > explosive.detonation_parameters.owner_safe_fuse
                    || player.id != explosive.owner_id
                {
                    explode = true;
                    break;
                }
            }
        }

        if explode {
            create_explosion(
                explosion_position,
                explosive.detonation_parameters.explosion_radius,
            );
            explosive.delete();
        }
    }

    fn draw(mut explosive: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        explosive.sprite.update();

        draw_texture_ex(
            resources.items_textures[&explosive.texture],
            explosive.body.pos.x,
            explosive.body.pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(explosive.sprite.frame().source_rect),
                dest_size: Some(explosive.sprite.frame().dest_size),
                flip_x: explosive.body.facing,
                rotation: 0.0,
                ..Default::default()
            },
        );
    }
}

/// Creates explosion FX and kills players within `radius`
pub fn create_explosion(position: Vec2, radius: f32) {
    let mut r = 0.0;
    {
        let mut resources = storage::get_mut::<Resources>();
        while r < radius - 5. {
            // Particles are 5 in radius
            r += 10.0 / (r + 1.);
            let angle = gen_range(0.0, f32::consts::PI * 2.);
            resources
                .fx_explosion_fire
                .emit(position + Vec2::new(angle.cos(), angle.sin()) * r, 1);
            //Explosion
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
