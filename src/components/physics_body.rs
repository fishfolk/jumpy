use macroquad::{color, experimental::collections::storage, prelude::*};

use macroquad_platformer::{Actor, World as CollisionWorld};

use crate::GameWorld;

pub struct PhysicsBody {
    pub collider: Actor,
    pub position: Vec2,
    pub size: Vec2,
    pub velocity: Vec2,
    pub is_facing_right: bool,
    pub rotation: f32,
    pub has_friction: bool,
    pub is_on_ground: bool,
    pub was_on_ground_last_frame: bool,
    pub has_gravity: bool,
    pub bouncyness: f32,
    pub can_rotate: bool,
    /// This is the offset between the collider and the body's position
    pub collider_offset: Vec2,
}

impl PhysicsBody {
    pub const GRAVITY: f32 = 1800.0;

    pub fn new<O: Into<Option<Vec2>>>(
        collision_world: &mut CollisionWorld,
        position: Vec2,
        angle: f32,
        size: Vec2,
        can_rotate: bool,
        has_friction: bool,
        collider_offset: O,
    ) -> PhysicsBody {
        let collider_offset = collider_offset.into().unwrap_or_default();

        let collider = collision_world.add_actor(position, size.x as _, size.y as _);

        PhysicsBody {
            position,
            size,
            is_facing_right: true,
            velocity: vec2(0., 0.),
            rotation: angle,
            has_friction,
            collider,
            was_on_ground_last_frame: false,
            is_on_ground: false,
            has_gravity: true,
            bouncyness: 0.0,
            can_rotate,
            collider_offset,
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
        let dt = get_frame_time();
        let mut world = storage::get_mut::<GameWorld>();

        // Don't use offset position for ground check
        let position = world.collision_world.actor_pos(self.collider);

        self.was_on_ground_last_frame = self.is_on_ground;
        self.is_on_ground = world
            .collision_world
            .collide_check(self.collider, position + vec2(0.0, 1.0));

        if !self.is_on_ground && self.has_gravity {
            self.velocity.y += Self::GRAVITY * dt;
        }

        if !world
            .collision_world
            .move_h(self.collider, self.velocity.x * dt)
        {
            self.velocity.x *= -self.bouncyness;
        }

        if !world
            .collision_world
            .move_v(self.collider, self.velocity.y * dt)
        {
            self.velocity.y *= -self.bouncyness;
        }

        if self.can_rotate {
            // TODO: Rotation
        }

        if self.is_on_ground && self.has_friction {
            self.velocity.x *= 0.96;
            if self.velocity.x.abs() <= 1.0 {
                self.velocity.x = 0.0;
            }
        }

        self.position = world.collision_world.actor_pos(self.collider) - self.collider_offset;
    }

    pub fn update_throw(&mut self) {
        if !self.is_on_ground {
            self.rotation += self.velocity.x.abs() * 0.00045 + self.velocity.y.abs() * 0.00015;

            self.velocity.y += Self::GRAVITY * get_frame_time();
        } else {
            self.rotation %= std::f32::consts::PI * 2.;
            let goal = if self.rotation <= std::f32::consts::PI {
                std::f32::consts::PI
            } else {
                std::f32::consts::PI * 2.
            };

            let rest = goal - self.rotation;
            if rest.abs() >= 0.1 {
                self.rotation += (rest * 0.1).max(0.1);
            }
        }

        self.velocity.x *= 0.96;
        if self.velocity.x.abs() <= 1. {
            self.velocity.x = 0.0;
        }
    }

    pub fn debug_draw(&self) {
        let position = self.position + self.collider_offset;

        draw_rectangle_lines(
            position.x,
            position.y,
            self.size.x,
            self.size.y,
            2.0,
            color::RED,
        )
    }

    pub fn get_collider_rect(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, self.size.x, self.size.y)
    }
}
