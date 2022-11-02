//! Systems and components related to damage/kill zones

use crate::{
    networking::proto::ClientMatchInfo,
    physics::{collisions::Rect, KinematicBody},
    player::{PlayerIdx, PlayerKillCommand},
    prelude::*,
};

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DamageRegion>()
            .register_type::<DamageRegionOwner>()
            .add_system_to_stage(
                FixedUpdateStage::PostUpdate,
                eliminate_players_in_damage_region,
            );
    }
}

/// A rectangular damage region.
///
/// While this _might_ change in the future, damage regions will kill players immediately, so there
/// is no "damage" field.
#[derive(Reflect, Debug, Clone, Component, Default)]
#[reflect(Component, Default)]
pub struct DamageRegion {
    /// The size of the damage region in pixels
    pub size: Vec2,
}

impl DamageRegion {
    /// Get the collision rectangle of this damage region, given it's transform.
    pub fn collider_rect(&self, transform: &Transform) -> Rect {
        Rect::new(
            transform.translation.x,
            transform.translation.y,
            self.size.x,
            self.size.y,
        )
    }
}

/// A component that may be added to a damage region entity to indicate the triggering entity.
///
/// If this entity is a player, it will not be harmed by the damage region.
///
/// Ideally this would probalby be a [`Option<Entity>`] field on [`DamageRegion`], but since scripts
/// can't interaction with Rust [`Option`]s yet, this is a solution that works now.
#[derive(Reflect, Debug, Clone, Component)]
#[reflect(Component, Default)]
pub struct DamageRegionOwner(pub Entity);

/// FIXME: Right now a [`Default`] implementation is required for scripts to be able to create
/// components of a certain type, but there isn't a great default value for [`Entity`] so we create
/// an entity with an index of [`u32::MAX`] as a workaround.
///
/// This workaround should be removed once scripts have a way to construct types without a
/// [`Default`] implementation.
impl Default for DamageRegionOwner {
    fn default() -> Self {
        Self(Entity::from_raw(u32::MAX))
    }
}

/// System that will eliminate players that are intersecting with a damage region.
fn eliminate_players_in_damage_region(
    mut commands: Commands,
    players: Query<(Entity, &PlayerIdx, &Transform, &KinematicBody)>,
    damage_regions: Query<(&DamageRegion, &Transform, Option<&DamageRegionOwner>)>,
    client_match_info: Option<Res<ClientMatchInfo>>,
) {
    for (player_ent, player_idx, player_transform, kinematic_body) in &players {
        // For network games, only consider the local player. We're not allowed to kill the other
        // players.
        if let Some(info) = &client_match_info {
            if player_idx.0 != info.player_idx {
                continue;
            }
        }

        let player_rect = kinematic_body.collider_rect(player_transform);
        for (damage_region, transform, damage_region_owner) in &damage_regions {
            // Don't damage the player that owns this damage region
            if let Some(owner) = damage_region_owner {
                if owner.0 == player_ent {
                    continue;
                }
            }

            let damage_rect = damage_region.collider_rect(transform);

            if player_rect.overlaps(&damage_rect) {
                commands.add(PlayerKillCommand::new(player_ent));
            }
        }
    }
}
