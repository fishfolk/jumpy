export default {
  preUpdate() {
    for (const entity of MapElement.getSpawnedEntities()) {
      let animated_sprite = Value.create(AnimatedSprite, {
        start: 0,
        end: 4,
        repeat: true,
        fps: 6,
        atlas: {
          id: Assets.getHandleId("./seaweed.atlas.yaml"),
        },
      });

      world.insert(entity, animated_sprite);
    }
  },
};
