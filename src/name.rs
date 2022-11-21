use crate::prelude::*;

pub struct NamePlugin;

impl Plugin for NamePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EntityName>()
            .add_system_to_stage(CoreStage::Last, update_entity_names)
            .extend_rollback_plugin(|plugin| plugin.register_rollback_type::<EntityName>());
    }
}

/// Conceptually identical to the [`Name`] component, but structured so that it can be added and
/// modified from scripts. Adding an [`EntityName`] component will cause a [`Name`] component to be
/// added and synced automatically.
#[derive(Reflect, Component, Default)]
#[reflect(Component, Default)]
pub struct EntityName(pub String);

fn update_entity_names(
    mut commands: Commands,
    names: Query<(Entity, &EntityName), Changed<EntityName>>,
) {
    for (entity, name) in &names {
        commands.entity(entity).insert(Name::new(name.0.clone()));
    }
}
