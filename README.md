# Fish Fight

[![Github Actions](https://github.com/fishfight/fish2/workflows/Compilation%20check/badge.svg)](https://github.com/fishfight/fish2/actions?query=workflow%3A)
![fish-scene](https://user-images.githubusercontent.com/583842/132137745-ee1f4565-bd75-4d56-b040-234a259ed2b7.gif)

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
- https://macroquad.rs/tutorials/fish-tutorial/
- https://not-fl3.github.io/platformer-book/intro.html
- https://sokoban.iolivia.me/ (we do not use an ECS)
- https://pragprog.com/titles/hwrust/hands-on-rust/ (ask Erlend for a free copy)

## Install & play

1. Install Rust with [Rustup.rs](https://rustup.rs/)
2. `cargo run`

We'll start distributing executables shortly!
