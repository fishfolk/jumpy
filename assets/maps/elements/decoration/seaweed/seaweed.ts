type AnimatedSprite = {
  start: usize;
  end: usize;
  atlas: HandleTextureAtlas;
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
      let animated_sprite = Value.create(AnimatedSprite, {
        start: 0,
        end: 4,
        repeat: true,
        fps: 6,
        atlas: {
          id: Assets.getHandleId(
            "../../../resources/default_decoration.atlas.yaml"
          ),
        },
      });

      world.insert(entity, animated_sprite);
    }
  },
};
