pub mod explosives {
    use macroquad::{
        color,
        experimental::{animation::AnimatedSprite, collections::storage, scene::RefMut},
        prelude::*,
    };

    use crate::{components::PhysicsBody, Resources};

    pub struct Explosive {
        texture: String,
        sprite: AnimatedSprite,
        body: PhysicsBody,
        lived: f32,
        owner_id: u8,
        detonation_parameters: DetonationParameters,
    }

    pub struct DetonationParameters {
        /// If a player is within this radius, it will explode. 0 for no proximity detonation. 
        pub trigger_radius: f32,
        /// Duration for which the owner of the explosive will not trigger the proximity fuze.
        pub owner_safe_fuse: f32,
        pub explosion_radius: f32,
        /// 0 for no automatic detonation.
        pub fuse: f32,
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
            let mut resources = storage::get_mut::<Resources>();

            let mut body = PhysicsBody::new(&mut resources.collision_world, pos, 0.0, size);
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

            if explosive.lived < explosive.detonation_parameters.fuse && explosive.detonation_parameters.fuse != 0. {
                if explosive.detonation_parameters.trigger_radius > 0. {
                    for player in crate::utils::player_circle_hit(
                        explosion_position,
                        explosive.detonation_parameters.trigger_radius,
                    ) {
                        if explosive.lived > explosive.detonation_parameters.owner_safe_fuse
                            || player.id != explosive.owner_id
                        {
                            explode = true;
                            break;
                        }
                    }
                }
            } else {
                explode = true;
            }

            if explode {
                crate::utils::explode(
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
}
