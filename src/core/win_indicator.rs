use crate::prelude::*;

/// Use value significantly father than map layers to make sure is in front
const WIN_INDICATOR_Z: f32 = -700.0;

#[derive(HasSchema, Clone, Default)]
#[type_data(metadata_asset("win_indicator"))]
#[repr(C)]
pub struct WinIndicatorMeta {
    pub attachment: Handle<AttachmentMeta>,
    pub sound: Handle<AudioSource>,
    pub volume: f32,
}

pub fn game_plugin(_: &mut Game) {
    WinIndicatorMeta::register_schema();
}

/// Command to spawn a win indicator and attach to entity
pub fn spawn_win_indicator(attach_entity: Entity) -> StaticSystem<(), ()> {
    (move |assets: Res<AssetServer>,
           mut audio_center: ResMut<AudioCenter>,
           mut entities: ResMut<Entities>,
           meta: Root<GameMeta>,
           mut transforms: CompMut<Transform>,
           mut atlas_sprites: CompMut<AtlasSprite>,
           mut attachments: CompMut<Attachment>,
           mut attachment_easings: CompMut<AttachmentEasing>| {
        let win_indicator_handle = meta.core.player_win_indicator;
        let win_indicator_meta = assets.get(win_indicator_handle);
        let attachment_meta = assets.get(win_indicator_meta.attachment);
        let easing_meta = assets.get(attachment_meta.attachment_easing);
        let win_entity = entities.create();

        // Compute z offset such that indicator is at target Z value
        let attach_transform = transforms.get(attach_entity).unwrap();
        let z_offset = WIN_INDICATOR_Z - attach_transform.translation.z;

        transforms.insert(win_entity, default());
        atlas_sprites.insert(
            win_entity,
            AtlasSprite {
                index: 0,
                atlas: attachment_meta.atlas,
                ..default()
            },
        );
        attachments.insert(
            win_entity,
            attachment_meta.attachment(attach_entity, z_offset),
        );

        let initial_offset = attachment_meta.offset;
        let target_offset = initial_offset + easing_meta.delta_offset;
        attachment_easings.insert(
            win_entity,
            AttachmentEasing {
                initial_offset,
                target_offset,
                offset_ease_timer: Timer::new(easing_meta.offset_ease_duration, TimerMode::Once),
                ease: Ease {
                    ease_in: easing_meta.ease.ease_in,
                    ease_out: easing_meta.ease.ease_out,
                    function: easing_meta.ease.function,
                    progress: 0.0,
                },
            },
        );

        audio_center.play_sound(win_indicator_meta.sound, win_indicator_meta.volume as f64);
    })
    .system()
}
