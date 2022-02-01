# Fish Fight

[![Build Status](https://img.shields.io/github/workflow/status/fishfight/FishFight/Compilation%20check?logo=github&labelColor=1e1c24&color=8bcfcf)](https://github.com/fishfight/FishFight/actions) [![Documentation](https://img.shields.io/badge/documentation-fishfight.github.io-green.svg?labelColor=1e1c24&color=f3ee7a)](https://fishfight.github.io/FishFight/) [![License](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg?label=license&labelColor=1e1c24&color=34925e)](./LICENSE) [![Discord](https://img.shields.io/badge/chat-on%20discord-green.svg?logo=discord&logoColor=fff&labelColor=1e1c24&color=8d5b3f)](https://discord.gg/4smxjcheE5)

![Fish Fight Preview](https://user-images.githubusercontent.com/24392180/151969075-399e9fea-e2de-4340-96a4-0a0e5b79c281.gif)

## Introduction

Fish Fight is a tactical 2D shooter, played by up to 4 players online or on a shared screen. Aim either left or right; the rest is up to clever movement and positioning in this fish-on-fish brawler! For more information about our origin story (Duck Game et.al.) and big-picture plans, see our [design document](https://www.notion.so/erlendsh/Fish-Fight-1647ed74217e4e38a59bd28f4f5bc81a).

### Key Features (WIP)

- 2 to 4 players in either Local Multiplayer or Online Play
- Easy to pick up, emphasizing strategy over twitch reaction
- Customize characters with hats, saved to your cross-platform profile
- Create & explore user-made weapons, levels, audio and other scripted extensions
- Smart level creation tools
- Tournaments & matchmaking built in

### Status

The game is fully playable: \
https://twitter.com/fishfightgame/status/1424084016467226624

## Community

### Contributing

Anyone involved in the Fish Fight community must follow our [code of conduct](https://github.com/fishfight/FishFight/blob/main/CODE_OF_CONDUCT.md).

If you'd like to make something for Fish Fight, check out our [help-wanted](https://github.com/fishfight/FishFight/labels/help%20wanted) issues or just ask us on [Discord](https://discord.gg/4smxjcheE5). We'll soon post an updated roadmap for the next month or two of work ahead.

Before committing and opening a PR, please run the following commands and follow their instructions:
1. `cargo clippy -- -W clippy::correctness -D warnings`
2. `cargo fmt`

### Learning Materials
- https://fishfight.github.io/FishFight/
- https://macroquad.rs/tutorials/fish-tutorial/ (outdated)
- https://not-fl3.github.io/platformer-book/intro.html (wip)
- https://cleancut.github.io/rusty_engine/
- https://sokoban.iolivia.me/
- https://pragprog.com/titles/hwrust/hands-on-rust/

## Download & play

1. Download the latest version from the [releases](https://github.com/fishfight/FishFight/releases) page.
2. Extract the archive and run the executable. (e.g. `./fishfight` or `fishfight.exe`)

### Launcher

[A cross-platform launcher](https://github.com/fishfight/Launcher) is also available for downloading and launching the game easily.

### Distro Packages

<details>
  <summary>Packaging status</summary>

[![Packaging status](https://repology.org/badge/vertical-allrepos/fishfight.svg)](https://repology.org/project/fishfight/versions)

</details>

#### Arch Linux

```sh
pacman -S fishfight
```

## Building

1. Install Rust with [rustup.rs](https://rustup.rs/)
2. Clone this repository: `git clone https://github.com/fishfight/FishFight.git`
3. `cargo run`

## Default key bindings

Keyboard left:
- movement: arrow keys `↑`, `←`, `↓`, `→`
- pick/drop: `K`
- attack: `L`

Keyboard right:
- movement: `W`, `A`, `S`, `D` (& `Space` for jump)
- pick/drop: `C`
- attack: `V` & `LeftCtrl`

Gamepad:
- movement: direction axis
- pick/drop: `X`
- attack: `B`
- jump: `A`
- slide: `Down` + `Y`

## Credits

- [FishFight Credits](./CREDITS.md)
- Input Icons: [Kadith's Icons](https://kadith.itch.io/kadiths-free-icons) by Kadith
