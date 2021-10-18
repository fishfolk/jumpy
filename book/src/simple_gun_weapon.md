# Simple Gun Weapon

This section will walk through the process of adding a new gun weapon to Fish Fight, using a sniper weapon as an example. The weapon added in this section will be an instance of the `Gun` struct.

## Planning

Before jumping into the games code, it is a good idea to do some planning about what you want your new weapon to do. I determined that my sniper weapon should have the following properties:

- High bullet speed
- Large recoil
- 2 bullets

## Programming

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

Next, create an `impl Gun` block and add a public function called `spawn_your_weapon`, for the sniper weapon I'm calling this function `spawn_sniper`. Then, add code to this function to spawn your new weapon. It should be very similar to the code I have here, but with your weapon's values instead of the sniper values. I copied the following code from `src/items/musket.rs` and changed the values to suit for my sniper weapon.

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
            resources.items_textures["sniper/gun"], // Change this to `your_weapon/gun`
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
            resources.items_textures["sniper/gun"], // Change this to `your_weapon/gun`
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
            // Use your weapon's constants here
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

### Items Array

Open `src/items.rs` and add find the `ITEMS` array. Add an entry to this array for your weapon using your new spawn function. I copied the `Item` entry for the musket weapon and changed the `tiled_name` and `constructor` fields to `sniper` and `gun::Gun::spawn_sniper` respectively. The `tiled_name` field is used identifying the item when designing levels. The `constructor` is the function to spawn the item into the game. We programmed this spawn function the previous [Item File](#item-file) section. The following is the entry for the sniper weapon:

```rust
Item {
    tiled_name: "sniper",
    constructor: gun::Gun::spawn_sniper,
    tiled_offset: (-35., -25.),
    textures: &[("gun", "assets/Whale/Gun(92x32).png")], // Temporarily using the existing gun texture
    sounds: &[],
    fxses: &[],
    network_ready: true,
},
```

If you like, you can skip to the testing section to test your new weapon, but if you haven't added a new texture or modified an existing texture, you will not be able to visually recognize your item before picking it up. For this reason, I recommend you continue to the [Texture](#texture) section next.

## Texture (Optional)

It is important to make sure that your weapon is able to be visually distinguished between the other weapons in the game. When I added the sniper rifle to the `ITEMS` array, I copied all of the data from existing musket weapon, changing only the `tiled_name` and `constructor`. To give my sniper weapon a new texture, I will also need to change the `gun` texture in the `textures` field. Currently, the `gun` texture is set to `assets/Whale/Gun(92x32).png`. I'll open this file with my pixel editor of choice, [Aseprite](https://www.aseprite.org/) ([GIMP](https://www.gimp.org/) would also work fine).

![open_gun_texture](assets/open_gun_texture.png)

I don't consider myself an artist so I'm just going to modify the hue of the golden part of the gun texture to a reddish color. First I'll select a color range and adjust the threshold to select only the golden part of the gun texture.

![select_gun_colors](assets/select_gun_colors.png)

Then I'll adjust the hue of the selected colors to turn all of the gold color red.

![adjust_gun_hue](assets/adjust_gun_hue.png)

This texture is now distinguishable from the other gun textures in the game. I'll save this new texture into the project's `assets/Whale` directory as `Sniper(92x32).png`.

This, of course, is just one way of distinguishing the texture from the other textures in the game. Feel free to copy and modify textures using your own methods, or if you feel inclined, make your own unique texture for your weapon!

Now all I have to do to put my new texture in the game is to change value of the gun texture for the sniper entry in the `ITEMS` array in `src/items.rs`. Here is the modified sniper entry using the new texture:

```rust
Item {
    tiled_name: "sniper",
    constructor: gun::Gun::spawn_sniper,
    tiled_offset: (-35., -25.),
    textures: &[("gun", "assets/Whale/Sniper(92x32).png")],
    sounds: &[],
    fxses: &[],
    network_ready: true,
},
```

## Size (Optional)

### Sprite

- sprites are defined in spawn function (sniper and musket)
- tiled height and width used as grid for sprites on spritesheet
- size of sprites can be modified as long as tiled width and height are updated as well
- explain gunlike animation with graphic (src/components/gunlike_animation.rs)

### Collider

- change collider width and collider height constants
- view hitboxes?

## Testing

The last thing we need to do is put our new weapon in the game and test it! Fish Fight's levels are defined in json files in the `assets/levels` directory. For testing items, there is a test level in the game defined in a file called `test_level.json`. Open this file.

In this file you will see a long list of item entries containing data about items that are placed in the level. The easiest way to add your new weapon to this level is to replace the `name` field of one of other items currently in the level with the name of your new weapon (referred to as `your_weapon` throughout this chapter). Here is the entry for my sniper weapon.

```json
...
{
    "draworder":"topdown",
    "id":5,
    "name":"items",
    "objects":[
        ...
        {
            "height":0,
            "id":147,
            "name":"sniper",
            "point":true,
            "rotation":0,
            "type":"",
            "visible":true,
            "width":0,
            "x":400,
            "y":690
        },
        ...
    ],
    ...
}
```

If you followed all of these steps correctly, your new weapon should be in the game. Run the game using `cargo run`, then select the test level. You should see your gun in the level and be able to try it out.

![sniper_weapon_test](assets/sniper_weapon_test.gif)

Now all you need to do is modify the values in `src/items/your_weapon.rs` until the weapon feels right to you! Then you are ready to make a pull request.
