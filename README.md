# Fish Fight

[![Github Actions](https://github.com/fishfight/fish2/workflows/Cross-compile/badge.svg)](https://github.com/fishfight/fish2/actions?query=workflow%3A)

### Introduction

Fish Fight is a tactical 2D shooter, played by up to 4 players online or on a shared screen. Aim either left or right; the rest is up to clever movement and positioning in this fish-on-fish brawler!

## Key Features

- 2 to 4 players in either Local Multiplayer or Online Play (currently focused on local)
- Easy to pick up, emphasizing strategy over twitch reaction
- Customize characters with hats, saved to your cross-platform profile
- Create & explore user-made weapons, levels, audio and other scripted extensions
- Smart level creation tools
- Tournaments & matchmaking built in

## The Plan

We are making a spiritual successor to the cult classic [Duck Game](https://store.steampowered.com/app/312530/Duck_Game/).

> Blast your friends with Shotguns, Net Guns, Mind Control Rays, Saxophones, Magnet Guns, and much, much more. This is DUCK GAME. Don't blink.

Duck Game is an excellent game. It deserves the 10/10 rating it can boast on Steam. But it is also stuck in an older tech stack. A combination of tech debt and lacking cross-platform infrastructure makes it very PC-centric. It's been ported to other platforms, albeit with a more limited experience; no online mods directory & delayed game updates.

The final update [still pending](https://twitter.com/superjoebob/status/1407628707754250241) for consoles is the last major update the game will ever receive.

There's ample opportunity for collaboration here: Fish Fight is a tribute to and continuation of Duck Game. *A spiritual successor*. To that end, we’ve been communicating with the creator of Duck Game (Landon) and got his blessing to make our own interpretation of his original game design.

### Status

The game is fully playable: \
https://twitter.com/fedor_games/status/1408868565772652544

#### Install

1. Install Rust with [Rustup.rs](https://rustup.rs/)
2. `cargo run`

We are currently focused on making the game as enjoyable as possible on local multiplayer, i.e. couch co-op. Check out the [roadmap](https://github.com/fishfight/fish2/issues/2) for more.

### Learning Materials
- https://macroquad.rs/tutorials/fish-tutorial/
- https://not-fl3.github.io/platformer-book/intro.html
- https://sokoban.iolivia.me/ (we do not use an ECS)
- https://pragprog.com/titles/hwrust/hands-on-rust/ (ask Erlend for a free copy)
