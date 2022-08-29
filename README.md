# Fish Folk: Jumpy

[![Build Status](https://img.shields.io/github/workflow/status/fishfolks/jumpy/Continuous%20integration?logo=github&labelColor=1e1c24&color=8bcfcf)](https://github.com/fishfolks/jumpy/actions) [![Documentation](https://img.shields.io/badge/documentation-fishfolks.github.io-green.svg?labelColor=1e1c24&color=f3ee7a)](https://fishfolks.github.io/jumpy/) [![License](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg?label=license&labelColor=1e1c24&color=34925e)](./LICENSE) [![Discord](https://img.shields.io/badge/chat-on%20discord-green.svg?logo=discord&logoColor=fff&labelColor=1e1c24&color=8d5b3f)](https://discord.gg/4smxjcheE5)

![Fish Folk: Jumpy Preview](https://user-images.githubusercontent.com/24392180/151969075-399e9fea-e2de-4340-96a4-0a0e5b79c281.gif)

## Introduction

Fish Folk: Jumpy is a tactical 2D shooter, played by up to 4 players online or on a shared screen. Aim either left or right; the rest is up to clever movement and positioning in this fish-on-fish brawler! For more information about our origin story (Duck Game et.al.) and big-picture plans, see our [design document](https://www.notion.so/erlendsh/Fish-Fight-1647ed74217e4e38a59bd28f4f5bc81a).

### Key Features (WIP)

- 2 to 4 players in either Local Multiplayer or Online Play
- Easy to pick up, emphasizing strategy over twitch reaction
- Customize characters with hats, saved to your cross-platform profile
- Create & explore user-made weapons, levels, audio and other scripted extensions
- Smart level creation tools
- Tournaments & matchmaking built in

### Status

The game is fully playable: \
https://twitter.com/spicylobsterfam/status/1424084016467226624

## Community

### Contributing

Anyone involved in the Fish Folk community must follow our [code of conduct](https://github.com/fishfolks/jumpy/blob/main/CODE_OF_CONDUCT.md).

If you'd like to make something for Fish Folk, check out our [help-wanted](https://github.com/fishfolks/jumpy/labels/help%20wanted) issues or just ask us on [Discord](https://discord.gg/4smxjcheE5). We'll soon post an updated roadmap for the next month or two of work ahead.

Before committing and opening a PR, please run the following commands and follow their instructions:
1. `cargo clippy -- -W clippy::correctness -D warnings`
2. `cargo fmt`

### Learning Materials
- https://fishfolks.github.io/jumpy/
- https://macroquad.rs/tutorials/fish-tutorial/ (outdated)
- https://not-fl3.github.io/platformer-book/intro.html (wip)
- https://cleancut.github.io/rusty_engine/
- https://sokoban.iolivia.me/
- https://pragprog.com/titles/hwrust/hands-on-rust/

## Download & play

1. Download the latest version from the [releases](https://github.com/fishfolks/jumpy/releases) page.
2. Extract the archive and run the executable. (e.g. `./jumpy` or `jumpy.exe`)

### Launcher

[A cross-platform launcher](https://github.com/spicylobstergames/SpicyLauncher) is also available for downloading and launching the game easily.

### Distro Packages

#### Arch Linux

```sh
pacman -S jumpy
```

## Building

1. Install Rust with [rustup.rs](https://rustup.rs/)
2. Clone this repository: `git clone https://github.com/fishfolks/jumpy.git`
3. `cargo run`

## Default Keybindings

You can view and modify the default keybindings in [config.toml](./config.toml).

## Credits

- [Fish Folk: Jumpy Credits](./CREDITS.md)
- Input Icons: [Kadith's Icons](https://kadith.itch.io/kadiths-free-icons) by Kadith
