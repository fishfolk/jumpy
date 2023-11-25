//! Damage / kill regions.
//!
//! Any player that intersects a damage region will be killed.

use crate::prelude::*;

use super::utils::Rect;

/// Install this module.
pub fn install(session: &mut Session) {
    DamageRegion::register_schema();
    DamageRegionOwner::register_schema();

    session
        .stages
        .add_system_to_stage(CoreStage::PostUpdate, kill_players_in_damage_region);
}

/// A rectangular damage region.
///
/// While this _might_ change in the future, damage regions will kill players immediately, so there
/// is no "damage" field.
#[derive(Debug, Clone, Default, HasSchema)]
#[repr(C)]
pub struct DamageRegion {
    /// The size of the damage region in pixels
    pub size: Vec2,
}

impl DamageRegion {
    /// Get the collision rectangle of this damage region, given it's transform.
    pub fn collider_rect(&self, position: Vec3) -> Rect {
        Rect::new(position.x, position.y, self.size.x, self.size.y)
    }
}

/// A component that may be added to a damage region entity to indicate the triggering entity.
///
/// If this entity is a player, it will not be harmed by the damage region.
// TODO: Make `DamageRegionOwner` a part of the `DamageRegion` component?
#[derive(Debug, Clone, HasSchema, Default)]
#[repr(C)]
pub struct DamageRegionOwner(pub Entity);

/// System that will eliminate players that are intersecting with a damage region.
fn kill_players_in_damage_region(
    entities: Res<Entities>,
    mut commands: Commands,
    player_indexes: Comp<PlayerIdx>,
    transforms: Comp<Transform>,
    damage_regions: Comp<DamageRegion>,
    damage_region_owners: Comp<DamageRegionOwner>,
    bodies: Comp<KinematicBody>,
    invincibles: CompMut<Invincibility>,
) {
    let mut bitset = player_indexes.bitset().clone();
    bitset.bit_and(transforms.bitset());
    bitset.bit_and(bodies.bitset());
    bitset.bit_andnot(invincibles.bitset());

    for player_ent in entities.iter_with_bitset(&bitset) {
        let transform = transforms.get(player_ent).unwrap();
        let body = bodies.get(player_ent).unwrap();

        let player_rect = body.bounding_box(*transform);
        for (ent, (damage_region, transform)) in entities.iter_with((&damage_regions, &transforms))
        {
            let owner = damage_region_owners.get(ent);
            // Don't damage the player that owns this damage region
            if let Some(owner) = owner {
                if owner.0 == player_ent {
                    continue;
                }
            }

            let damage_rect = damage_region.collider_rect(transform.translation);
            if player_rect.overlaps(&damage_rect) {
                commands.add(PlayerCommand::kill(
                    player_ent,
                    Some(transform.translation.xy()),
                ));
            }
        }
    }
}
