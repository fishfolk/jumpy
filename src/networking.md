Networked multi-player plugin.

Jumpy uses a Peer-to-Peer, rollback networking model built on [GGRS].

Messages are serialized/deserialized to a binary representation using [`serde`] and the [`postcard`]
crate.

The major facets of our networking are:

- **[Matchmaking](#matchmaking):** How we connect clients to each-other and start an network match.
- **[Synchronization](#synchronization):** How we synchronize a network game between multiple players.

[ggrs]: https://github.com/gschup/ggrs
[`serde`]: https://docs.rs/serde
[`postcard`]: https://docs.rs/postcard

## Matchmaking

There are currently two different matchmaking strategies: [`online`] and [`lan`]. Both of those
modules contain their own matchmaker with docs on how it works. Eventually we will probably have
additional matchmakers for Steam and the browser.

Regardless of the matchmaker, the goal is to find a match and establish a connection to the other
players. Once a match is established, the matchmaker must provide an implementation of
[`NetworkSocket`] that may be used to send [GGRS], reliable, and unreliable messages.

Each matchmaker is free to implement this socket with whatever networking transport they wish,
allowing the Steam matchmaker, for example, to use the steam networking library, and the browser
matchmaker to use `WebTransport` or `WebRTC`.

## Synchronization

Match synchronization, as mentioned above, is accomplished with [GGRS], wich is a re-imagining of
the [GGPO] network SDK.

The [`NetworkSocket`] trait, which matchmakers are required to implement for their sockets, is
required to return an implementation of GGRS's [`NonBlockingSocket`] trait, so that it can send it's
unreliable messages. The exact method for sending these messages depends on the matchmaker.

The key requirement for rollback networking is:

- The synchronized game loop must be **deterministic**.
- We must have the ability to **snapshot** and **restore** the game state.
- We must be able to run up to 8 game simulation frames in 16ms ( to achieve a 60 FPS frame rate,
  even in the case where we have to rollback and re-simulate ).

These requirements are provided for primarily by [`bones_lib`]. The core match logic is implemented
in [`jumpy_core::session::CoreSession`][::jumpy_core::session::CoreSession], allowing us to advance
the game, snapshot the game, and restore it.

The integration with GGRS is implemented by the [`GgrsSessionRunner`].

[`NonBlockingSocket`]: https://docs.rs/ggrs/0.9.2/ggrs/trait.NonBlockingSocket.html
[ggpo]: https://github.com/pond3r/ggpo/tree/master
[`bones_lib`]: https://fishfolk.github.io/bones/rustdoc/bones_lib/index.html

## Development & Debugging

Here are some tips for debugging networking features while developing.

### Local Sync Test

It can be cumbersome to start a new networked match every time you need to troubleshoot some part of
the game that may not be rolling back or restoring properly. To help with this, you can run the game
with the `--sync-test-check-distance 7` to make the game test rolling back and forward 8 times every
frame as a test when starting a local game.

This allows you to test the rollback without having to connect to a server. If things start popping
around the map or having other weird behaviors that they don't have without the sync-test mode, then
you know you have a rollback issue.

> **ℹ️ Note:** Just because you **don't** have an issue in sync test mode, doesn't mean that there
> is no determinism issues. You still have to test network games with multiple game instances. There
> are some non-determinism issues that only exhibit themselves when restarting the game.
