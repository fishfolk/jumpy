//! Systems and components related to damage/kill zones

use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PostUpdate, kill_players_in_damage_region);
}

/// A rectangular damage region.
///
/// While this _might_ change in the future, damage regions will kill players immediately, so there
/// is no "damage" field.
#[derive(Debug, Clone, Default, TypeUlid)]
#[ulid = "01GP1X5MBXZNEC4Y0WF5AKCA3Z"]
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
#[derive(Debug, Clone, TypeUlid)]
#[ulid = "01GP1X4NM7GMEKKZ4FEZ1RK3T0"]
pub struct DamageRegionOwner(pub Entity);

/// System that will eliminate players that are intersecting with a damage region.
fn kill_players_in_damage_region(
    entities: Res<Entities>,
    player_indexes: Comp<PlayerIdx>,
    transforms: Comp<Transform>,
    damage_regions: Comp<DamageRegion>,
    damage_region_owners: Comp<DamageRegionOwner>,
    bodies: Comp<KinematicBody>,
    mut player_events: ResMut<PlayerEvents>,
) {
    for (player_ent, (_idx, transform, body)) in
        entities.iter_with((&player_indexes, &transforms, &bodies))
    {
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
                player_events.kill(player_ent);
            }
        }
    }
}
