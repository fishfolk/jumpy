# Particles
FishFight uses macroquad's [macroquad-particles](https://crates.io/crates/macroquad-particles) for simulating and drawing particles. While it might seem simple, it is deceptively so. With a little creativity plenty impressive effects can be created!

Also see the [the particle section of the juice page.](../juice.md#particles)

### The components of a particle effect
A particle effect consists of several parts that must be set up as following:

First, you need to design your effect! Each effect has a `json` file that describes how the system spawns, looks, flies, brakes and fades. Check out some of the other effects  in `assets/particles/`, or easier, try [Fedor's tool](https://fedorgames.itch.io/macroquad-particles) with previews and live tweaking to generate the file for you. 

Now that you got your **particle system definition**, give it a descriptive name and put it in `assets/particle_effects/` with the others. Then open `assets/particle_effects.json` (a file, not a directory this time) and add your particle effect to it. Preferably use the same ID as your filename, excluding the extension. This tells the game to load your effect into the `Resources` object, so you can actually access it.

### Spawning a particle
The easiest way to use your particle is through an effect that spawns a particle system. Just supply the ID of your effect in the JSON-definition of your object and the item's code will handle the rest. It can look something like this, depending on the effect:

```json
"effects": [
	...
	{
		"type": "particle_controller",
		"id": "0",
		"particle_id": "musket_muzzle_smoke",
		"start_delay": 1,
		"is_can_be_interrupted": true,
		"amount": 6,
		"interval": 0.05
	},
	...
]
```

#### Spawning in code
Soon you'll probably want to trigger particle effects from your own code. You do that as following:
```rust
scene::find_node_by_type::<ParticleEmitters>().unwrap().spawn("particle_id_here", position: Vec2);
```
If you want to spawn multiple systems, you can store a reference:
```rust
let mut particles = scene::find_node_by_type::<ParticleEmitters>().unwrap();
```

You can also use a `ParticleController` which manages spawning particles over time for you, useful for leaving trails (this is currently used for the smoke puffs left behind bullets). Its `new`-constructor takes a `ParticleControllerParameters`, where you can control spawn rates and such. It is documented in the code. Note that the `ParticleController` has an `update()` function that has to be called each frame to make it tick.