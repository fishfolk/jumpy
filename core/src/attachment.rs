//! Utilities for attaching entities to other entities.

use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    session
        .stages
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
                .or_else(|| {
                    sprites
                        .get_mut(attachment.entity)
                        .map(|x| (&mut x.flip_x, &mut x.flip_y))
                })
            {
                *self_flip_x = flip_x;
                *self_flip_y = flip_y;
            }
        }

        transform.translation += offset;
    }
}
