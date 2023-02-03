//! Utilities for attaching entities to other entities.

use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::Last, update_player_body_attachments)
        .add_system_to_stage(CoreStage::Last, remove_player_body_attachments)
        .add_system_to_stage(CoreStage::Last, update_attachments);
}

/// Component for attaching an entity to another entity.
///
/// > **Warning:** attachments are not a general-purpose hierarchy system. Generally, only expect an
/// > attachment to work properly if the entity it is attached to is not also attached to another
/// > entity.
///
/// Attachments have special behavior built-in for attaching to other [`Sprite`] or [`AtlasSprite`]
/// entities. When attached to a sprite, the attached item will automatically flip it's offset when
/// the sprite is flipped, and will also synchronize it's own flip value, if it has a sprite component.
#[derive(Clone, Copy, TypeUlid, Debug)]
#[ulid = "01GQJQ07Z0S07B7EWDA4HW938Q"]
pub struct Attachment {
    /// The entity to attach to.
    pub entity: Entity,
    /// The offset to the attached entity.
    pub offset: Vec3,
    /// Synchronize [`AtlasSprite`] animation with entity animation
    pub sync_animation: bool,
}

/// System to update the transforms of entities with the [`Attachment`] component.
pub fn update_attachments(
    entities: Res<Entities>,
    mut transforms: CompMut<Transform>,
    attachments: Comp<Attachment>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut sprites: CompMut<Sprite>,
) {
    for (ent, attachment) in entities.iter_with(&attachments) {
        let Some(attached_transform) = transforms.get(attachment.entity).copied() else {
            continue;
        };
        let transform = transforms
            .get_mut(ent)
            .expect("Entities with `Attachment` component must also have a `Transform` component.");

        *transform = attached_transform;

        let mut offset = attachment.offset;
        if let Some((flip_x, flip_y)) = atlas_sprites
            .get(attachment.entity)
            .map(|x| (x.flip_x, x.flip_y))
            .or_else(|| sprites.get(attachment.entity).map(|x| (x.flip_x, x.flip_y)))
        {
            if flip_x {
                offset.x *= -1.0;
            }
            if flip_y {
                offset.y *= -1.0;
            }

            if let Some((self_flip_x, self_flip_y)) = atlas_sprites
                .get_mut(ent)
                .map(|x| (&mut x.flip_x, &mut x.flip_y))
                .or_else(|| sprites.get_mut(ent).map(|x| (&mut x.flip_x, &mut x.flip_y)))
            {
                *self_flip_x = flip_x;
                *self_flip_y = flip_y;
            }
        }

        if attachment.sync_animation {
            if let Some((index, attach_atlas)) = atlas_sprites
                .get(attachment.entity)
                .map(|atlas| atlas.index)
                .zip(atlas_sprites.get_mut(ent))
            {
                attach_atlas.index = index
            }
        }

        transform.translation += offset;
    }
}

/// A component for attaching an entity to the player's body.
///
/// This is similar to the [`Attachment`] component, but it is special in the way that it will
/// follow the body as it bobs up and down in animations such as standing and walking. This makes it
/// useful for things like hats, etc., that will stick to the player's body.
#[derive(Clone, TypeUlid)]
#[ulid = "01GQQSZS823YZS2RBAPFNBKB8B"]
pub struct PlayerBodyAttachment {
    /// The player to attach to
    pub player: Entity,
    /// The offset relative to the center of the player's sprite.
    pub offset: Vec3,
    /// Whether or not to automatically play the same animation bank animation as the sprite that it
    /// is attached to.
    pub sync_animation: bool,
}

#[derive(Clone, Copy, TypeUlid)]
#[ulid = "01GQQWDPHAJKNM686ZY425V4XF"]
pub struct HadPlayerBodyAttachmentMarker;

fn update_player_body_attachments(
    entities: Res<Entities>,
    mut attachments: CompMut<Attachment>,
    mut player_body_attachment_markers: CompMut<HadPlayerBodyAttachmentMarker>,
    animated_sprites: Comp<AnimatedSprite>,
    animation_banks: Comp<AnimationBankSprite>,
    player_body_attachments: Comp<PlayerBodyAttachment>,
    player_inputs: Res<PlayerInputs>,
    player_indexes: Comp<PlayerIdx>,
    player_assets: BevyAssets<PlayerMeta>,
) {
    for (ent, body_attachment) in entities.iter_with(&player_body_attachments) {
        let player_ent = body_attachment.player;
        let player_idx = player_indexes.get(player_ent).unwrap();
        let player_handle = &player_inputs.players[player_idx.0].selected_player;
        let Some(meta) = player_assets.get(&player_handle.get_bevy_handle()) else {
            continue;
        };
        let player_sprite = animated_sprites.get(player_ent).unwrap();
        let current_frame = player_sprite.index;
        let current_anim = animation_banks.get(player_ent).unwrap().current;

        let current_body_offset =
            meta.layers.body.animations.body_offsets[&current_anim][current_frame];

        player_body_attachment_markers.insert(ent, HadPlayerBodyAttachmentMarker);
        attachments.insert(
            ent,
            Attachment {
                entity: player_ent,
                offset: current_body_offset.extend(0.0) + body_attachment.offset,
                sync_animation: body_attachment.sync_animation,
            },
        );
    }
}

fn remove_player_body_attachments(
    entities: Res<Entities>,
    player_body_attachments: Comp<PlayerBodyAttachment>,
    mut had_player_body_attachment_markers: CompMut<HadPlayerBodyAttachmentMarker>,
    mut attachments: CompMut<Attachment>,
) {
    let mut bitset = had_player_body_attachment_markers.bitset().clone();
    bitset.bit_andnot(player_body_attachments.bitset());

    for entity in entities.iter_with_bitset(&bitset) {
        attachments.remove(entity);
        had_player_body_attachment_markers.remove(entity);
    }
}
