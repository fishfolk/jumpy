Networked multi-player plugin.

Jumpy uses a Peer-to-Peer, rollback networking model built on [GGRS] and the [`bevy_ggrs`] plugin.

We use a centralized matchmaking server to connect peers to each-other and to forward the peers'
network traffic. All connections utilize UDP and the QUIC protocol.

Messages are serialized/deserialized to a binary representation using [`serde`] and the [`postcard`]
crate.

The major facets of our networking are:

- [Matchmaking](#matchmaking): How we connect clients to each-other and start an online match.
- [Synchronization](#synchronization): How we synchronize a network game between multiple players.

You may also want to see:

- [Future Changes](#future-changes) for some thoughts on changes we
    might make to the current design.
- [Development & Debuggin](#development--debugging) for tips on testing networking during
    development.

[ggrs]: https://github.com/gschup/ggrs
[`bevy_ggrs`]: https://github.com/gschup/bevy_ggrs
[`serde`]: https://docs.rs/serde
[`postcard`]: https://docs.rs/postcard

## Matchmaking

In order to establish the peer connections we use a matchmaking server implemented in the
[`bones_matchmaker`] crate. This server binds one UDP port and listens for client connections.
Because QUIC supports mutliplexing connections, we are able to handle any number of clients on a
single UDP port.

All client traffic is proxied to other peers through the matchmaking server. In this way it is not
true peer-to-peer networking, but logically, once the match starts, clients are sending messages to
each-other, and the server doesn't take part in the match protocol.

Having the matchmaker proxy client messages has the following pros and cons:

**Cons:**

- It uses up more of the matchmaking server's bandwidth
- It adds an extra network hop between peers, increasing latency.

**Pros:**

- It reduces the number of connections each peer needs to make. Each peer only holds one
  connection to the matchmaking server and nothing else.
- It hides the IP addresses of clients from each-other. This is an important privacy feature.
- It avoids a number of difficulties that you may run into while trying to establish true
  peer-to-peer connections, and makes it much easier to bypass firewalls, NATs, etc.

This doesn't prevent us from supporting true peer-to-peer connections in the future, though.
Similarly, another scenario we will support in the future is LAN games that you can join without
needing a matchmaking server.

[`bones_matchmaker`]: https://github.com/fishfolk/bones/tree/main/crates/bones_matchmaker

### Matchmaking Protocol

> **ℹ️ Note:** This is meant as an overview and is not an exact specification of the matchmaking
> protocol.

#### Initial Connection

When a client connects to the matchmaking server, the very first thing it will do is send a
[`RequestMatch`][bones_matchmaker_proto::MatchmakerRequest::RequestMatch] message to the server over
a reliable channel.

This message contains the [`MatchInfo`][`bones_matchmaker_proto::MatchInfo`] that tells the server
how many players the client wants to connect to for the match, along with an arbitrary byte sequence
for the `match_data`.

In order for players to end up in the same match as each-other, they must specify the _exact_ same
`MatchInfo`, including the `match_data`. We use the `match_data` as a way to specify which game mode
and parameters, etc. that the player wants to connect to, so that all the players that are connected
to each-other are playing the same mode.

The `match_data` also contains the game name and version. Because the matchmaker does not take part
in the match protocol itself, just the matchmaking protocol, **this makes the matchmaking server
game agnostic**. Different games can connect to the same matchmaking server, and they can make sure
they are only connected to players playing the same game, by specifying a unique `match_data`.

> **Note:** To be clear, the game implementation sets the `match_data` for players. Players are
> never exposed directly to the concept of the `match_data`.

#### Waiting For Players

After the initial connection and match request, the server will send the client an
[`Accepted`][`bones_matchmaker_proto::MatchmakerResponse::Accepted`] message.

If the waiting room for that match already has the desired number of players in it, the server will
then respond immediately with a [`Success`][bones_matchmaker_proto::MatchmakerResponse::Success]
message. This message comes with:

- a `random_seed` that can be used by all clients to generate deterministic random numbers, and
- a `player_idx` that tells the client _which_ player in the match it is. This is used throughout
    the game to keep track of the players, and is between `0` and `player_count - 1`.

#### In the Match

Immediately after the desired number of clients have joined and the `Success` message has been sent
to all players, the matchmaker goes into proxy mode for all clients in the match.

Once in proxy mode, the server listens for
[`SendProxyMessage`][`bones_matchmaker_proto::SendProxyMessage`]s from clients. Each message simply
specifies a [`TargetClient`][bones_matchmaker_proto::TargetClient] ( either a specific client or all
of them ), and a binary message data.

Once it a `SendProxyMessage` it will send it to the target client, which will receive it in the form
of a [`RecvProxyMessage`][bones_matchmaker_proto::RecvProxyMessage], containing the message data,
and the index of the client that sent the message.

The matchmaking server supports forwarding both reliable and unreliable message in this way,
allowing the game to chose any kind of protocol it sees fit to synchronize the match data.

## Synchronization

Match synchronization, as mentioned above, is accomplished with [GGRS], wich is a re-imagining of
the [GGPO] network SDK.

We implement GGRS's [`NonBlockingSocket`] trait on top of our QUIC `Connection`, using QUIC's raw
datagram feature to send messages unreliably. This way we can proxy all of the GGRS traffic through
the matchmaking server, while still allowing it to use it's own reliability protocol.

All of the Bevy systems that need to be synchronized during a match are added to their own Bevy
[Schedule][bevy::ecs::schedule::Schedule]. We use an [extension
trait][crate::schedule::RollbackScheduleAppExt] on the Bevy [`App`][bevy::app::App] to make it
easier to add systems to the rollback schedule in our plugins throughout Jumpy.

The key requirement for rollback networking is:

- The synchronized game loop must be **deterministic**.
- We must have the ability to **snapshot** and **restore** the game state.
- We must be able to run up to 8 game simulation frames in 16ms ( to achieve a 60 FPS frame rate ).

[`NonBlockingSocket`]: https://docs.rs/ggrs/0.9.2/ggrs/trait.NonBlockingSocket.html

### Determinism

Luckily, Jumpy's physics and game logic is simple and we don't face any major non-determinism
issues. The primary source of potential non-determinism is Bevy's query iteration order and entity
allocation.

#### Sorting Queries

Because Bevy doesn't guarantee any specific order for entity iteration, we have to manually collect
and sort queries when a different order could produce a different in-game result.

For all rollback entities, we can simply collect the query results into a [`Vec`] and sort by the
[`Rollback`] component's id, but sometimes we have non-rollback entities such as the map element
spawners that we also need to iterate over. For those we have a simple [`Sort`][crate::utils::Sort]
component that just stores an index. We get that index from something deterministic, such as the
order that the map elements appear in the map YAML file.

It's easy to accidentally forget to sort entities when querying, and you may not notice issues until
you try to run a network game, and the clients end up playing a "different version" of the same
game. We hope we can improve this: see [Future Changes](#future-changes).

[ggpo]: https://www.ggpo.net/

#### Spawning Entities

When spawning entities, we need to attach [`Rollback`][bevy_ggrs::Rollback] components to them, that
contain a unique index identifying the entity across rollbacks and restores, which may modify the
Entity's entity ID.

We must be careful every time we spawn an item, that we deterministically assign the same `Rid` to
the entity on all clients. This mostly boils down to making sure we spawn them in the same order.

### Snapshot & Restore

All of the components that need to be synchronized during rollback must be registered with the
[`bevy_ggrs`] plugin. This is usually done in the Bevy plugin that adds the component, by calling
[`extend_rollback_plugin()`][crate::schedule::RollbackScheduleAppExt::extend_rollback_plugin] using
the extension trait on the Bevy `App` type.

The [`bevy_ggrs`] plugin will then make sure that that component is snapshot and restored during
rollback and restore.

Currently [`bevy_ggrs`] requires a [`Reflect`][bevy::reflect::Reflect] implementation on components
that will be synchronized, and it uses the `Reflect` implementation to clone the objects. We have
noticed that snapshot and restore using this technique can take up to 1ms. There are already plans,
once Bevy lands it's "Stageless" implementation, to re-implement [`bevy_ggrs`] and remove the
`Reflect` requirement, which should improve snapshot performance.

This is important because it is hard to fit 8 frames into a 16ms time period, and taking a whole 1ms
to snapshot cuts down on how many frames we can run in that period of time.

## Future Changes

These are some ideas for future changes to the networking strategy.

### Encapsulate Core Match Logic in an Isolated Micro ECS

In order to improve our determinism and snapshot/restore story, we are
discussing ( see [#489] and [#510] ) an alternative architecture for handling the synchronization of
the match state.

The idea is to move the core match game loop into it's own, tiny ECS that doesn't have the
non-deterministic iteration order problem, and that can also be snapshot and restored simply by
copying the entire ECS world.

This creates a healthy isolation between Bevy and it's various resources and entities, and our core
game loop. Additionally, we may put this isolated ECS in a WASM module to allow for hot reloading
core game logic, and enabling mods in the future.

[#489]: https://github.com/fishfolk/jumpy/discussions/489
[#510]: https://github.com/fishfolk/jumpy/discussions/510

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
