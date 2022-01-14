use macroquad::{
    audio::play_sound_once,
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, HandleUntyped, RefMut},
        state_machine::{State, StateMachine},
    },
    prelude::*,
};

use crate::Resources;

pub struct Sproinger;

impl Sproinger {
    pub const OBJECT_ID: &'static str = "sproinger";

    pub const TRIGGER_WIDTH: f32 = 32.0;
    pub const TRIGGER_HEIGHT: f32 = 8.0;
    pub const FORCE: f32 = 1100.0;pub fn update_normal(node: &mut RefMut<Self>, _dt: f32) {
        let sproinger_rect = Rect::new(
            node.pos.x, // - (Self::TRIGGER_WIDTH / 2.0),
            node.pos.y + (node.sprite.frame().dest_size.y - Self::TRIGGER_HEIGHT),
            Self::TRIGGER_WIDTH,
            Self::TRIGGER_HEIGHT,
        );

        for physics_object in scene::find_nodes_with::<PhysicsObject>().filter(|obj| obj.active()) {
            let object_collider = physics_object.collider();
            let intersect = sproinger_rect.intersect(object_collider);
            if intersect.is_some() {
                let resources = storage::get_mut::<Resources>();
                let jump_sound = resources.sounds["jump"];

                play_sound_once(jump_sound);

                physics_object.set_speed_y(-Self::FORCE);

                node.state_machine.set_state(Self::ST_JUMP);
            }
        }
    }

    pub fn update_jump(_node: &mut RefMut<Self>, _dt: f32) {}

    fn jump_coroutine(node: &mut RefMut<Self>) -> Coroutine {
        let node_handle = node.handle();

        let coroutine = async move {
            {
                let mut node = scene::get_node(node_handle);
                node.sprite.set_animation(2);
            }
            for i in 0..2 {
                {
                    let mut node = scene::get_node(node_handle);
                    if node.sprite.current_animation() != 2 {
                        return;
                    }
                    node.sprite.set_frame(i);
                }
                wait_seconds(0.08).await;
            }
            wait_seconds(0.5).await;

            {
                let mut node = scene::get_node(node_handle);
                node.sprite.set_animation(0);
                node.state_machine.set_state(Self::ST_NORMAL);
            }
        };
        start_coroutine(coroutine)
    }
}

impl Sproinger {
    fn network_capabilities() -> NetworkReplicate {
        fn network_update(handle: HandleUntyped) {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Sproinger>();
            Sproinger::network_update(node);
        }

        NetworkReplicate { network_update }
    }

    fn network_update(node: RefMut<Self>) {
        StateMachine::update_detached(node, |node| &mut node.state_machine);
    }
}

impl scene::Node for Sproinger {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::network_capabilities());
    }

    fn draw(mut node: RefMut<Self>) {
        node.sprite.update();

        let resources = storage::get::<Resources>();
        let texture_entry = resources.textures.get("sproinger").unwrap();

        draw_texture_ex(
            texture_entry.texture,
            node.pos.x,
            node.pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.sprite.frame().source_rect),
                dest_size: Some(node.sprite.frame().dest_size),
                flip_x: false,
                rotation: 0.0,
                ..Default::default()
            },
        );
    }
}
