## Packaging Fish Fight

### Dependencies

#### Build dependencies

- [Rust](https://www.rust-lang.org/tools/install) (`1.53.0+`)
- [CMake](https://cmake.org/download/) (only needed if `bundled-sdl2` feature is **enabled** in [Cargo.toml](./Cargo.toml))
- [SDL2](https://www.libsdl.org/download-2.0.php) (only needed if `bundled-sdl2` feature is **disabled** in [Cargo.toml](./Cargo.toml))

#### Runtime dependencies

##### Linux

- [libX11](https://gitlab.freedesktop.org/xorg/lib/libx11)
- [libXi](https://gitlab.freedesktop.org/xorg/lib/libxi)
- [Mesa](https://www.mesa3d.org/) - [OpenGL](https://www.opengl.org) (`3.2+`)
- [alsa-lib](https://github.com/alsa-project/alsa-lib)

Also see [macroquad#linux](https://github.com/not-fl3/macroquad#linux).

### Build

```sh
# export CARGO_TARGET_DIR=target
cargo run --release --locked
```

### Environment variables

- `FISHFIGHT_CONFIG`: configuration file (default: `config.json`)
- `FISHFIGHT_ASSETS`: assets directory (default: `assets/`)
- `FISHFIGHT_MODS`:  mods directory (default: `mods/`)

### Package

Binary will be located at `target/release/fishfight` after [build](#build). To run it, `assets` directory should be placed in the same directory or a path can be specified via `FISHFIGHT_ASSETS` environment variable.

For example:

```sh
export FISHFIGHT_ASSETS=/opt/fishfight/assets/
export FISHFIGHT_MODS=/opt/fishfight/mods/
target/release/fishfight
```

Also see [README.md#distro-packages](./README.md#distro-packages)

### Binary releases

Binary releases are automated via [Continuous Deployment](./.github/workflows/release.yml) workflow and they can be downloaded from the [releases](https://github.com/fishfight/FishFight/releases) page. Release artifacts are named in the following format:

- `fishfight-<version>-<target>.<ext>`

A single archive includes the `fishfight` binary and `assets` directory. It can be verified by using a SHA256 hash file that has the same name as the artifact except it ends with ".sha256". Release artifacts are not signed at this time.
