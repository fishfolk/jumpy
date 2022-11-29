/**
 * TODO: Migrate to Rust
 */

type MenuBackground = unknown;
const MenuBackground: BevyType<MenuBackground> = {
  typeName: "jumpy::ui::main_menu::MainMenuBackground",
};

export default {
  update() {
    const time = world.resource(Time);
    const query = world.query(MenuBackground, Transform);
    for (const item of query) {
      const [_, transform] = item.components;

      let scale = 1.5 + Math.sin(time.seconds_since_startup * 0.8) * 0.2;
      transform.scale.x = scale;
      transform.scale.y = scale;

      let offset = Math.sin(time.seconds_since_startup * 0.4) * 50;
      transform.translation.x = offset + 60;
      transform.translation.y = offset * 0.2 - 20;
    }
  },
};
