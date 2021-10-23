use macroquad::{
    experimental::collections::storage,
    math::{vec2, Vec2},
    time::get_frame_time,
};

use macroquad_platformer::{Actor, World as CollisionWorld};

use crate::GameWorld;

pub struct PhysicsBody {
    pub pos: Vec2,
    pub size: Vec2,
    pub velocity: Vec2,
    pub is_facing_right: bool,
    pub angle: f32,
    pub has_friction: bool,
    pub collider: Actor,
    pub on_ground: bool,
    pub last_frame_on_ground: bool,
    pub has_gravity: bool,
    pub bouncyness: f32,
    pub can_rotate: bool,
}

impl PhysicsBody {
    pub const GRAVITY: f32 = 1800.0;

    pub fn new(
        collision_world: &mut CollisionWorld,
        pos: Vec2,
        angle: f32,
        size: Vec2,
        can_rotate: bool,
        has_friction: bool,
    ) -> PhysicsBody {
        PhysicsBody {
            pos,
            size,
            is_facing_right: true,
            velocity: vec2(0., 0.),
            angle,
            has_friction,
            collider: collision_world.add_actor(pos, size.x as _, size.y as _),
            last_frame_on_ground: false,
            on_ground: false,
            has_gravity: true,
            bouncyness: 0.0,
            can_rotate,
        }
    }

    pub fn facing_dir(&self) -> Vec2 {
        if self.is_facing_right {
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
        self.on_ground = world
            .collision_world
            .collide_check(self.collider, self.pos + vec2(0., 1.));

        if !self.on_ground && self.has_gravity {
            self.velocity.y += Self::GRAVITY * get_frame_time();
        }

        if !world
            .collision_world
            .move_h(self.collider, self.velocity.x * get_frame_time())
        {
            self.velocity.x *= -self.bouncyness;
        }

        if !world
            .collision_world
            .move_v(self.collider, self.velocity.y * get_frame_time())
        {
            self.velocity.y *= -self.bouncyness;
        }

        self.pos = world.collision_world.actor_pos(self.collider);

        if self.can_rotate {
            // TODO: Rotation
        }

        if self.on_ground && self.has_friction {
            self.velocity.x *= 0.96;
            if self.velocity.x.abs() <= 1.0 {
                self.velocity.x = 0.0;
            }
        }
    }

    pub fn update_throw(&mut self) {
        if !self.on_ground {
            self.angle += self.velocity.x.abs() * 0.00045 + self.velocity.y.abs() * 0.00015;

            self.velocity.y += Self::GRAVITY * get_frame_time();
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

        self.velocity.x *= 0.96;
        if self.velocity.x.abs() <= 1. {
            self.velocity.x = 0.0;
        }
    }
}
