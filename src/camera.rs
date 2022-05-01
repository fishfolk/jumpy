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

        if rect.x + rect.width > player.camera_box.x + player.camera_box.width {
            player.camera_box.x = rect.x + rect.width - player.camera_box.width;
        }

        if rect.y < player.camera_box.y {
            player.camera_box.y = rect.y;
        }

        if rect.y + rect.height > player.camera_box.y + player.camera_box.height {
            player.camera_box.y = rect.y + rect.height - player.camera_box.height;
        }

        player_rects.push(rect);
    }

    let mut camera = main_camera();
    camera.update(&player_rects);

    Ok(())
}

#[cfg(not(feature = "macroquad"))]
pub fn update_camera(_world: &mut World, _delta_time: f32) -> Result<()> {
    Ok(())
}
