# Simple Gun Weapon

This section will walk through designing and programming a sniper rifle gun weapon. This weapon will be a instance of a `Gun` struct.

## Planning

The sniper rifle weapon will have:

- High bullet speed
- Large recoil
- 2 bullets

## Code

Open your cloned "FishFight" directory using your code editor of choice.

### Item File

Open `src/items.rs` and add a new module called `your_weapon` at the top of the file. Below is the code for adding the sniper module:

```rust
mod sniper;
```

Create a new rust file in `src/items` called `your_weapon.rs`. I'll name the item file for this example `sniper.rs`.

At the top of the file, add the following imports:

```rust
use macroquad::{
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene::{self, HandleUntyped},
    },
    prelude::*,
};

use crate::{
    components::{GunlikeAnimation, PhysicsBody, ThrowableItem},
    items::gun::Gun,
    Resources,
};
```

Next, define some constant values for your new item. The following are the values that I came up with for the sniper rifle, but you should play around with your own values until you find values that you like for your item.

```rust
const SNIPER_COLLIDER_WIDTH: f32 = 48.0;
const SNIPER_COLLIDER_HEIGHT: f32 = 32.0;
const SNIPER_RECOIL: f32 = 1400.0;
const SNIPER_BULLETS: i32 = 2;
const SNIPER_BULLET_SPEED: f32 = 1200.0;
```

Next, create an `impl Gun` block and add a public function called `spawn_your_item`, for the sniper weapon I'm calling this function `spawn_sniper`. Then add code to this function to spawn your new weapon. It should be very similar to the code I have here, but with my your weapon's values instead of the sniper values.

```rust
impl Gun {
    pub fn spawn_sniper(pos: Vec2) -> HandleUntyped {
        let mut resources = storage::get_mut::<Resources>();

        let gun_sprite = GunlikeAnimation::new(
            AnimatedSprite::new(
                92,
                32,
                &[
                    Animation {
                        name: "idle".to_string(),
                        row: 0,
                        frames: 1,
                        fps: 1,
                    },
                    Animation {
                        name: "shoot".to_string(),
                        row: 1,
                        frames: 3,
                        fps: 15,
                    },
                ],
                false,
            ),
            resources.items_textures["musket/gun"],
            SNIPER_COLLIDER_WIDTH,
        );

        let gun_fx_sprite = GunlikeAnimation::new(
            AnimatedSprite::new(
                92,
                32,
                &[Animation {
                    name: "shoot".to_string(),
                    row: 2,
                    frames: 3,
                    fps: 15,
                }],
                false,
            ),
            resources.items_textures["sniper/gun"],
            SNIPER_COLLIDER_WIDTH,
        );

        scene::add_node(Gun {
            gun_sprite,
            gun_fx_sprite,
            gun_fx: false,
            body: PhysicsBody::new(
                &mut resources.collision_world,
                pos,
                0.0,
                vec2(SNIPER_COLLIDER_WIDTH, SNIPER_COLLIDER_HEIGHT),
            ),
            throwable: ThrowableItem::default(),
            bullets: SNIPER_BULLETS,
            max_bullets: SNIPER_BULLETS,
            bullet_speed: SNIPER_BULLET_SPEED,
            collider_width: SNIPER_COLLIDER_WIDTH,
            collider_height: SNIPER_COLLIDER_HEIGHT,
            recoil: SNIPER_RECOIL,
        })
        .untyped()
    }
}
```

## Items Array

Open `src/items.rs` and add find the `ITEMS` array. Add an entry to this array for your item using your new spawn function. The following is the entry for the sniper weapon:

```rust
Item {
    tiled_name: "sniper",
    constructor: gun::Gun::spawn_sniper,
    tiled_offset: (-35., -25.),
    textures: &[("gun", "assets/Whale/Gun(92x32).png")],
    sounds: &[],
    fxses: &[],
    network_ready: true,
},
```

- `src/main.rs`: Add resources to Resources?

## Assets

`assets/Whale`: Add textures for weapon

## Testing

- `levels/test_level.json`: Add item to test level
