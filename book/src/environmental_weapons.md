# Environmental weapons

- [Environmental weapons](#environmental-weapons)
  - [Document notes](#document-notes)
  - [Mechanics and examples](#mechanics-and-examples)
  - [Code design](#code-design)

Environmental weapons are weapons that don't follow the structure of weapon and projectile, rather, they spawn one or more entities intended to harm enemy.

## Document notes

This document doesn't explain all the concepts, since some are already explained in [Physics](physics.md).

`EW` is used as abbreviation for `E`nvironmental `W`eapon.

## Mechanics and examples

For each pickup, it's possible to spawn the weapon only once; after usage, most will disappear entirely from the level.

Currently, there are a few EWs:

- Curse: a skull that chases the closest enemy, in a sinusoidal motion, for a limited amount of time;
- Galleon: a large vessel that crosses the screen horizontally;
- Shark rain: a group of sharks falling vertically from the top, at random horizontal positions.

EWs may, or may not, self-pwn the owner.

## Code design

EWs are typically designed in two main types: the item, which is what the player can pick up/hold/throw, and the weapon itself, which is the set of entities that are summoned and harm the enemies.

The general design is very similar to all the other weapons, with a few distinctions.

The `shoot()` routine, itself common to the other weapons, is a good starting point:

```rs
pub fn shoot(galleon: Handle<Galleon>, player: Handle<Player>) -> Coroutine {
    let coroutine = async move {
        /* ... */

        if galleon.used {
            player.state_machine.set_state(Player::ST_NORMAL);
            return;
        }

        galleon.used = true;

        FlyingGalleon::spawn(player.id);

        player.weapon = None;
        
        /* ... */

        galleon.delete();

        player.state_machine.set_state(Player::ST_NORMAL);
    };

    start_coroutine(coroutine)
}
```

A very important concept is that we must avoid race conditions on multiple shots. Since shooting is asynchronous, on the first shot, we need to set a flag (in this case, `used`) that makes other concurrent executions return.

Most EWs are usable only once per level; this is implemented by removing the weapon from the player (see above), and deleting the item from the node graph.

Since most of the EWs don't kill the owner, we store the Player id in the spawned type, and skip it on collision test:

```rs
pub struct FlyingGalleon {
    /* ... */
    owner_id: u8,
}

impl scene::Node for FlyingGalleon {
    fn fixed_update(mut flying_galleon: RefMut<Self>) {
        /* ... */

        for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
            if player.dead {
                continue;
            }

            if flying_galleon.owner_id != player.id {
                /* Collision test here */
```

We also check if a player is dead, before performing the the collision check: since the EWs are generally large, performing the test multiple times is not correct; for example, it has the immediate effect of causing multiple death effects.

EWs typically need to know the boundaries of the map; see the `FlyingGalleon#start_position_data()` routine:

```rs
fn start_position_data() -> (Vec2, bool) {
    let resources = storage::get::<Resources>();

    // We query the map size from the raw tiled map data:

    let map_width =
        resources.tiled_map.raw_tiled_map.tilewidth * resources.tiled_map.raw_tiled_map.width;
    let map_height =
        resources.tiled_map.raw_tiled_map.tileheight * resources.tiled_map.raw_tiled_map.height;

    // Note also how we generate a random boolean via MacroQuad `rand` APIs:

    let (start_x, direction) = if gen_range(0., 1.) < 0.5 {
        (0. - FLYING_GALLEON_WIDTH, true)
    } else {
        ((map_width - 1) as f32, false)
    };

    let start_y = gen_range(0., map_height as f32 - FLYING_GALLEON_HEIGHT);

    (vec2(start_x, start_y), direction)
}
```
