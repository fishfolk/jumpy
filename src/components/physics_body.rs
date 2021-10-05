use macroquad::{
    experimental::collections::storage,
    math::{vec2, Vec2},
    time::get_frame_time,
};

use macroquad_platformer::{Actor, World as CollisionWorld};

use crate::{GameWorld};

pub struct PhysicsBody {
    pub pos: Vec2,
    pub size: Vec2,
    pub speed: Vec2,
    pub facing: bool,
    pub angle: f32,
    pub collider: Actor,
    pub on_ground: bool,
    pub last_frame_on_ground: bool,
    pub have_gravity: bool,
    pub bouncyness: f32,
}

impl PhysicsBody {
    pub const GRAVITY: f32 = 1800.0;

    pub fn new(
        collision_world: &mut CollisionWorld,
        pos: Vec2,
        angle: f32,
        size: Vec2,
    ) -> PhysicsBody {
        PhysicsBody {
            pos,
            size,
            facing: true,
            speed: vec2(0., 0.),
            angle,
            collider: collision_world.add_actor(pos, size.x as _, size.y as _),
            last_frame_on_ground: false,
            on_ground: false,
            have_gravity: true,
            bouncyness: 0.0,
        }
    }

    pub fn facing_dir(&self) -> Vec2 {
        if self.facing {
            vec2(1., 0.)
        } else {
            vec2(-1., 0.)
        }
    }

    pub fn descent(&mut self) {
        let mut world = storage::get_mut::<GameWorld>();
        world.collision_world.descent(self.collider);
    }

    pub fn update(&mut self) {
        let mut world = storage::get_mut::<GameWorld>();

        self.pos = world.collision_world.actor_pos(self.collider);
        self.last_frame_on_ground = self.on_ground;
        self.on_ground = world.collision_world.collide_check(self.collider, self.pos + vec2(0., 1.));
        if !self.on_ground && self.have_gravity {
            self.speed.y += Self::GRAVITY * get_frame_time();
        }
        if !world.collision_world.move_h(self.collider, self.speed.x * get_frame_time()) {
            self.speed.x *= -self.bouncyness;
        }
        if !world.collision_world.move_v(self.collider, self.speed.y * get_frame_time()) {
            self.speed.y *= -self.bouncyness;
        }
        self.pos = world.collision_world.actor_pos(self.collider);
    }

    pub fn update_throw(&mut self) {
        if !self.on_ground {
            self.angle += self.speed.x.abs() * 0.00045 + self.speed.y.abs() * 0.00015;

            self.speed.y += Self::GRAVITY * get_frame_time();
        } else {
            self.angle %= std::f32::consts::PI * 2.;
            let goal = if self.angle <= std::f32::consts::PI {
                std::f32::consts::PI
            } else {
                std::f32::consts::PI * 2.
            };

            let rest = goal - self.angle;
            if rest.abs() >= 0.1 {
                self.angle += (rest * 0.1).max(0.1);
            }
        }

        self.speed.x *= 0.96;
        if self.speed.x.abs() <= 1. {
            self.speed.x = 0.0;
        }
    }
}
