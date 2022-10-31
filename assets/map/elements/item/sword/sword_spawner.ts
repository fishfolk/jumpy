export default {
  preUpdateInGame() {
    const spawnedEntities = MapElement.getSpawnedEntities();

    // Handle newly spawned map entities
    for (const spanwer_entity of spawnedEntities) {
      const [transform, global_transform, computed_visibility] = world
        .query(Transform, GlobalTransform, ComputedVisibility)
        .get(spanwer_entity);

      // Spawn a new entity for the sword and copy the transform and visibility from the map element
      const entity = world.spawn();
      world.insert(
        entity,
        Value.create(Item, {
          script: Assets.getAbsolutePath("./sword.ts"),
        })
      );
      world.insert(entity, transform);
      world.insert(entity, global_transform);
      world.insert(entity, computed_visibility);
      world.insert(entity, Value.create(Visibility));
    }
  },
};
