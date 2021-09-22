# Physics

As you might already know, Fish Fight is a homage to the excellent Duck Game and thus we have chosen to replicate the rather simple and straight forward linear physics found in that game. It follows standard platformer physics, with force being translated to two-dimensional velocity, acting against gravity and drag, every physics update (`fixed_update`).

[The same physics system is used in Towerfall and Celeste.](https://maddythorson.medium.com/celeste-and-towerfall-physics-d24bd2ae0fc5)

To learn more about basic game physics, as they are implemented in Fish Fight, you can explore the following sources:  

`[Ask us about missing links!]`

## Fish Fight's physics implementation

As for the specifics of Fish Fight, I will elaborate in the following paragraphs. Please note, however, that the game is currently in a very early prototype stage, so the implementation is neither perfect nor, in any way, optimized, at this stage.

### Scene nodes

The game uses the [Macroquad library](https://github.com/not-fl3/macroquad), by [Fedor](https://github.com/not-fl3), who is also part of the core team of Fish Fight. This means that our scenes are composed of scene nodes, made by implementing the `Node` type. The most relevant method, when discussing physics, is the `fixed_update` method, which is called for every node, every physics frame. This means that, in order to explore the existing physics of any existing in-game object, you should browse to the corresponding node source file, in [`src/nodes`](https://github.com/fishfight/fish2/tree/master/src/nodes), and look for the `impl Node for T` section and the encapsulated `fixed_update` implementation.  

This method takes a `RefMut<T>` as an argument (can be both mutable and immutable), where `T` is the type of the node that it is being implemented for. From here, you can do many things; like manipulating the node, itself, through the `RefMut<T>` parameter, as well as access other nodes by fetching them from the scene, either by type, or by specific traits, made by calling `node.provides([...])` in a nodes `ready` implementation. For examples of this, you can check the code of most nodes for the code providing the `Sproingable` trait, for example.

Examples of code for accessing other nodes:

```rust
// This is from the ArmedGrenade node, showing how we iterate over players, checking for
// collision and killing the player if the explosion "collides" with the player
for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
    let intersect =
        grenade_rect.intersect(Rect::new(
            player.body.pos.x,
            player.body.pos.y,
            PLAYER_HITBOX_WIDTH,
            PLAYER_HITBOX_HEIGHT,
        ));
    if !intersect.is_none() {
        let direction = node.body.pos.x > (player.body.pos.x + 10.);
        scene::find_node_by_type::<crate::nodes::Camera>()
            .unwrap()
            .shake();
        player.kill(direction);
    }
}
```

```rust
// This is from the Sproinger node, iterating through nodes providing the Sproingable trait
// and checking for collision, before performing a "sproing" on them, if they overlap
for (_actor, mut body_lens, size) in scene::find_nodes_with::<Sproingable>() {
    if body_lens.get().is_some() {
        let body = body_lens.get().unwrap();
        if body.speed.length() > Self::STOPPED_THRESHOLD {
            let intersect = sproinger_rect
                .intersect(Rect::new(body.pos.x, body.pos.y, size.x, size.y));
            if !intersect.is_none() {
                let resources = storage::get_mut::<Resources>();
                play_sound_once(resources.jump_sound);

                body.speed.y = -Self::FORCE;
                node.has_sproinged = true;
                // self.sprite.set_animation(1);
                // self.sprite.playing = true;
                Sproinger::animate(node.handle());
            }
        }
    }
}
```

### Collision

Collision between nodes is done by creating collider `Rect` or `Circle` objects and calling their `intersect` (`Rect` only) or `overlaps` methods. The former will return an`Option<Rect>`, where the contained `Rect` represents the intersection between the two colliding `Rect` objects, or `None`, if there was no intersection. The latter will return a `bool` that is `true` if there was any overlap between the two objects. To check for collisions with the map, you have several methods in the `scene` module that lets you check for collisions on the various map layers. For examples of map collisions, once again, the [`Player`](https://github.com/fishfight/fish2/tree/master/src/nodes/player.rs) implementation is a good place to start. Furthermore, `PhysicsBody` members may hold a collider that Macroquads physics engine will collide against `Solid` objects in the scene (ground tiles, for the most part). These colliders will have to be added to the collision world, as actors. See the constructor of the [`player node`](https://github.com/fishfight/fish2/tree/master/src/nodes/player.rs) for an example of how this is done.

NOTE: To get the hitbox of a Player node, use `Player::get_hitbox()`, so that you get the correct size if the player node, for example, is in a crouched state.

### Force

When it comes to enacting force on nodes, this is done by setting a speed on a node. Most nodes will have a body, but not all, as the primary use for a body is to hold a collider. For simpler nodes, we might just put a position vector and a speed vector directly on the node, in stead. This can be checked in the specific nodes implementation. As mentioned, we set the velocity directly, in stead of accumulating force over several frames, as this leads to much more predictable and precise game physics (the age old Mario, using acceleration, vs Megaman, using binary force, dichotomy).  

This means that, in order to implement an explosion, for example, you would decide on a force, find the node(s) that the explosion will act upon and apply this force by setting the speed of the node, or the nodes body, depending on its implementation, to the value of the force in the appropriate direction. See below for a simplified example of something exerting a force, on the x-axis, on a player, if an arbitrary hit check condition is fulfilled:

```rust
impl T {
    pub const FORCE: f32 = 900.0;
}

impl Node for T {
    for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
        [hit detection]
        if is_hit {
            let direction_is_right = node.body.pos.x > (player.body.pos.x + 10.);
            player.body.speed.x = if direction_is_right {
                FORCE
            } else {
                -FORCE // negative force
            }
        }
    }
}
```

The same methods would be used for movement, for example, but instead of checking for a collision, you would check for input. For examples of this, check out the [`player node implementation`](https://github.com/fishfight/fish2/tree/master/src/nodes/player.rs).
