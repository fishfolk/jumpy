Contains the online matchmaker and the [`NetworkSocket`] implementation.

## Matchmaking

For online matchmaking, we use a centralized matchmaking server to connect peers to each-other and
to forward the peers' network traffic. All connections utilize UDP and the QUIC protocol.

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
[`RequestMatch`][crate::external::bones_matchmaker_proto::MatchmakerRequest::RequestMatch] message
to the server over a reliable channel.

This message contains the [`MatchInfo`][crate::external::bones_matchmaker_proto::MatchInfo] that
tells the server how many players the client wants to connect to for the match, along with an
arbitrary byte sequence for the `match_data`.

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
[`Accepted`][crate::external::bones_matchmaker_proto::MatchmakerResponse::Accepted] message.

If the waiting room for that match already has the desired number of players in it, the server will
then respond immediately with a
[`Success`][crate::external::bones_matchmaker_proto::MatchmakerResponse::Success] message. This
message comes with:

- a `random_seed` that can be used by all clients to generate deterministic random numbers, and
- a `player_idx` that tells the client _which_ player in the match it is. This is used throughout
    the game to keep track of the players, and is between `0` and `player_count - 1`.

#### In the Match

Immediately after the desired number of clients have joined and the `Success` message has been sent
to all players, the matchmaker goes into proxy mode for all clients in the match.

Once in proxy mode, the server listens for
[`SendProxyMessage`][crate::external::bones_matchmaker_proto::SendProxyMessage]s from clients. Each
message simply specifies a [`TargetClient`][crate::external::bones_matchmaker_proto::TargetClient] (
either a specific client or all of them ), and a binary message data.

Once it a `SendProxyMessage` it will send it to the target client, which will receive it in the form
of a [`RecvProxyMessage`][crate::external::bones_matchmaker_proto::RecvProxyMessage], containing the
message data, and the index of the client that sent the message.

The matchmaking server supports forwarding both reliable and unreliable message in this way,
allowing the game to chose any kind of protocol it sees fit to synchronize the match data.
