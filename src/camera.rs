use ff_core::prelude::*;

use crate::player::Player;

#[cfg(feature = "macroquad")]
pub fn update_camera(world: &mut World, _delta_time: f32) -> Result<()> {
    let mut player_rects = Vec::new();

    for (_, (transform, player)) in world.query_mut::<(&Transform, &mut Player)>() {
        let rect = Rect::new(transform.position.x, transform.position.y, 32.0, 60.0);

        if rect.x < player.camera_box.x {
            player.camera_box.x = rect.x;
        }

        if rect.x + rect.w > player.camera_box.x + player.camera_box.w {
            player.camera_box.x = rect.x + rect.w - player.camera_box.w;
        }

        if rect.y < player.camera_box.y {
            player.camera_box.y = rect.y;
        }

        if rect.y + rect.h > player.camera_box.y + player.camera_box.h {
            player.camera_box.y = rect.y + rect.h - player.camera_box.h;
        }

        player_rects.push(rect);
    }

    let mut camera = active_camera();
    camera.update(&player_rects);

    Ok(())
}

#[cfg(not(feature = "macroquad"))]
pub fn update_camera(_world: &mut World, _delta_time: f32) -> Result<()> {
    Ok(())
}
