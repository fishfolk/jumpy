type AnimatedSprite = {
  start: usize;
  end: usize;
  atlas_path: string;
  flip_x: boolean;
  flip_y: boolean;
  repeat: boolean;
  fps: f32;
};
const AnimatedSprite: BevyType<AnimatedSprite> = {
  typeName: "jumpy::animation::AnimatedSprite",
};

export default {
  preUpdate() {
    for (const entity of MapElement.getSpawnedEntities()) {
      world.insert(
        entity,
        Value.create(AnimatedSprite, {
          start: 5,
          end: 9,
          repeat: true,
          fps: 6,
          atlas_path: Assets.absolutePath("../../../resources/default_decoration.atlas.yaml"),
        })
      );
    }
  },
};
