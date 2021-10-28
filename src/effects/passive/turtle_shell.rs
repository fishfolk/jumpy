use macroquad::{
    experimental::{
        coroutines::{start_coroutine, Coroutine},
        scene::Handle,
    },
    prelude::*,
};

use crate::player::{Player, PlayerEventParams};

pub const COROUTINE_ID: &str = "turtle_shell";

pub fn coroutine(
    instance_id: &str,
    item_id: Option<&str>,
    player_handle: Handle<Player>,
    event_params: PlayerEventParams,
) -> Coroutine {
    let instance_id = instance_id.to_string();
    let item_id = item_id.map(|str| str.to_string());

    let coroutine = async move {
        if let Some(mut node) = scene::try_get_node(player_handle) {
            if let PlayerEventParams::ReceiveDamage { is_from_right, .. } = event_params {
                if node.body.is_facing_right == is_from_right {
                    node.kill(is_from_right);
                } else if item_id.is_some() {
                    let mut is_depleted = false;

                    if let Some(instance) = node.passive_effects.get(&instance_id) {
                        if let Some(uses) = instance.uses {
                            if uses == instance.use_cnt {
                                is_depleted = true;
                            }
                        }
                    } else {
                        is_depleted = true;
                    }

                    let item_id = item_id.unwrap();

                    if is_depleted {
                        node.equipped_items.remove(&item_id);
                    } else if let Some(item) = node.equipped_items.get_mut(&item_id) {
                        if let Some(sprite) = item.sprite_animation.as_mut() {
                            sprite.set_frame(1);
                        }
                    }
                }
            }
        }
    };

    start_coroutine(coroutine)
}
