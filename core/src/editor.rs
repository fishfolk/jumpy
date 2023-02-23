use crate::{map::z_depth_for_map_layer, prelude::*};

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, handle_editor_input);
}

fn handle_editor_input(
    player_inputs: Res<PlayerInputs>,
    mut entities: ResMut<Entities>,
    mut spawned_map_meta: ResMut<SpawnedMapMeta>,
    mut element_handles: CompMut<ElementHandle>,
    mut transforms: CompMut<Transform>,
    mut spawned_map_layer_metas: CompMut<SpawnedMapLayerMeta>,
) {
    for player in &player_inputs.players {
        if let Some(editor_input) = &player.editor_input {
            match editor_input {
                EditorInput::SpawnElement {
                    handle,
                    translation,
                    layer,
                } => {
                    let ent = entities.create();
                    element_handles.insert(ent, ElementHandle(handle.clone()));
                    let z_depth = z_depth_for_map_layer(*layer as usize);
                    transforms.insert(
                        ent,
                        Transform::from_translation(translation.extend(z_depth)),
                    );
                    spawned_map_layer_metas.insert(
                        ent,
                        SpawnedMapLayerMeta {
                            layer_idx: *layer as usize,
                        },
                    );
                }
                EditorInput::CreateLayer { id } => {
                    let ent = entities.create();
                    let layer_idx = spawned_map_meta.layer_names.len();
                    spawned_map_meta.layer_names = spawned_map_meta
                        .layer_names
                        .clone()
                        .iter()
                        .cloned()
                        .chain([id.clone()])
                        .collect();
                    spawned_map_layer_metas.insert(ent, SpawnedMapLayerMeta { layer_idx });
                }
                EditorInput::MoveEntity { entity, pos } => {
                    let transform = transforms.get_mut(*entity).unwrap();
                    transform.translation.x = pos.x;
                    transform.translation.y = pos.y;
                }
                EditorInput::DeleteEntity { entity } => {
                    entities.kill(*entity);
                }
            }
        }
    }
}
