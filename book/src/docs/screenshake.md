# Screen Shake
Screen shake in FishFight is accessed through the Camera node in the scene. It has multiple functions to apply different kinds of noise.

To camera node is accessed as follows:
```rust
let mut camera = scene::find_node_by_type::<crate::game::GameCamera>().unwrap();
```

Unless you use the camera multiple times, it's better to not assign it to a variable but to use the value straight away: 
```rust
let mut camera = scene::find_node_by_type::<crate::game::GameCamera>().unwrap().shake_noise(magnitude, length, frequency);
```
This prevents some ownership issues that would otherwise require separate scopes.

### Parameters
`magnitude`: How far the shake moves the screen in pixels. Values around 10-20 are sane.

`length`: For how long the shake will last in frames. The magnitude is multiplied by `age/length` to fade the shake over time.

`frequency`: 1 is normal, 0.2 is a punch. With a frequency of 0.2 and length of 10 frames it will oscillate max twice. 0.5 is more of a rumble. Worth noting is that `shake_sinusodial` has a different base frequency.

[See this](../juice.md#screen-shake-in-practice-in-fish-fight) for more information.

### Noise types
```rust
camera.shake_noise(magnitude: f32, length: i32, frequency: f32);
```
Shakes the screen around randomly using Perlin noise. Applicable for almost anything. 

```rust
camera.shake_noise_dir(magnitude: f32, length: i32, frequency: f32, direction: (f32, f32));
```
Like `shake_noise`, but the X and Y components of the resulting shake will be multiplied by the direction tuple. Can be used to make the screen shake more on one axis than the other, or only on one axis.

```rust
camera.shake_sinusoidal(magnitude: f32, length: i32, frequency:f32, angle: f32);
```
Shakes the screen sinusoidally along the angle given in radians. 

```rust
camera.shake_rotational(magnitude: f32, length: i32);
```
Rotates the scene around the screen's centre. Combines well with other types of screen shake for extra punchiness.