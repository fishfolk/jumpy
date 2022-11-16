//! Systems and components related to damage/kill zones

use crate::{
    physics::{collisions::Rect, KinematicBody},
    player::{PlayerKillCommand, PlayerIdx},
    prelude::*,
};

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DamageRegion>()
            .register_type::<DamageRegionOwner>()
            .extend_rollback_plugin(|plugin| {
                plugin
                    .register_rollback_type::<DamageRegion>()
                    .register_rollback_type::<DamageRegionOwner>()
            })
            .extend_rollback_schedule(|schedule| {
                schedule.add_system_to_stage(
                    RollbackStage::PostUpdate,
                    kill_players_in_damage_region,
                );
            });
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
    pub fn collider_rect(&self, position: Vec3) -> Rect {
        Rect::new(position.x, position.y, self.size.x, self.size.y)
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
fn kill_players_in_damage_region(
    mut commands: Commands,
    players: Query<(Entity, &GlobalTransform, &KinematicBody), With<PlayerIdx>>,
    damage_regions: Query<(&DamageRegion, &GlobalTransform, Option<&DamageRegionOwner>)>,
) {
    for (player_ent, player_global_transform, kinematic_body) in &players {
        let player_rect = kinematic_body.collider_rect(player_global_transform.translation());
        for (damage_region, global_transform, damage_region_owner) in &damage_regions {
            // Don't damage the player that owns this damage region
            if let Some(owner) = damage_region_owner {
                if owner.0 == player_ent {
                    continue;
                }
            }

            let damage_rect = damage_region.collider_rect(global_transform.translation());

            if player_rect.overlaps(&damage_rect) {
                commands.add(PlayerKillCommand::new(player_ent));
            }
        }
    }
}
