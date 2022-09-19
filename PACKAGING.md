## Packaging Fish Folk: Jumpy

### Dependencies

#### Build dependencies

- [Rust](https://www.rust-lang.org/tools/install) (`1.53.0+`)
- [libudev](https://www.freedesktop.org/software/systemd/man/libudev.html)

#### Runtime dependencies

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

- `JUMPY_CONFIG`: configuration file (default: `config.json`)
- `JUMPY_ASSETS`: assets directory (default: `assets/`)
- `JUMPY_MODS`: mods directory (default: `mods/`)

### Package

Binary will be located at `target/release/jumpy` after [build](#build). To run it, `assets` directory should be placed in the same directory or a path can be specified via `JUMPY_ASSETS` environment variable.

For example:

```sh
export JUMPY_ASSETS=/opt/jumpy/assets/
export JUMPY_MODS=/opt/jumpy/mods/
target/release/jumpy
```

The desktop file in the contrib/ directory can be installed to allow running the game from your desktop's app launcher.

Also see [README.md#distro-packages](./README.md#distro-packages)

### Binary releases

Binary releases are automated via [Continuous Deployment](./.github/workflows/release.yml) workflow and they can be downloaded from the [releases](https://github.com/fishfolks/jumpy/releases) page. Release artifacts are named in the following format:

- `jumpy-<version>-<target>.<ext>`

A single archive includes the `jumpy` binary and `assets` directory. It can be verified by using a SHA256 hash file that has the same name as the artifact except it ends with ".sha256". Release artifacts are not signed at this time.
