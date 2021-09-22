# Netcode

The netcode in [Fish Fight](https://github.com/fishfight/FishFight) is something called delay-based netcode by many players, but is more  specifically deterministic lockstep. A peer-to-peer architecture that is completely deterministic.

This video was the main inspiration for our netcode architecture:

<iframe src="https://www.youtube.com/embed/7jb0FOcImdg" allowfullscreen="" width="500" height="281" frameborder="0"></iframe>

What is described from 6:09 to 7:15 is the current state of our  netcode. The video proceeds to explain the limitations of this approach  and how they iterated from there to what can more accurately be called [GGPO](https://en.wikipedia.org/wiki/GGPO). That is where we are headed as well, beginning with a minimal  foundation that we have a full grasp and agency over. From the  intentionally naive “Player Unhappy” model we will gradually work our  way towards “Player Very Happy”.

## Terms and Concepts

Before we go any further, let’s explain some terms and concepts.

First off, what makes online multiplayer games difficult to make?  There are two main issues: (1) The latency (travel time) for information over the internet, and (2) when circumventing that, ensuring that  nobody can cheat while also fairly integrating everyone’s input. What’s  important isn’t that the solution is ”correct”, but that it feels great  to your players. Game networking consists largely of clever tricks and  smoke and mirrors, all to disguise the inherent limits of space-time.

A peer/client is someone who joins an online game session. (We will  use these terms interchangeably, but there might be reasons to  differentiate the two). For example, when you play Minecraft, you join  servers as a client. Most games these days use a server-client  architecture, where clients will join a server that handles everything  in the online game session, and streams necessary data to the clients.

However, this isn’t the only way of handling things. Indeed, before  the server-client architecture became commonplace, there was the  peer-to-peer (P2P) architecture. Instead of relying on an authoritative  server to tell the clients what’s happening, the peers instead tell EACH OTHER what is happening. This means that any one peer has to send out  data to every single other peer, and as such the required bandwidth  scales linearly as more players join. For this reason most games just  use the server-client system for anything more than 6-8 players.  However, for a game like Fish Fight which will only have 2-4 players in  any one given match, a peer-to-peer system will make more sense.

This has several benefits. First and foremost, there’s no need to pay for hosting. While for a small project like Fish Fight, with a  relatively simple simulation, that cost would likely be low. But there’s a plethora of problems that must be addressed: that cost isn’t zero and still has to be paid somehow, and if the game gets immensely popular  the price will quickly skyrocket. And then you must also write a load  distribution system, and DDOS protection, the list goes on. A  peer-to-peer architecture offloads all the costs to the players, who are in any case already running the simulation. This also means that you  don’t need to maintain two separate client and server codebases, that  must exactly line up in behaviour. Peer-to-peer might even offer lower  latency, since the packets don’t have to go back and forth through a  server.

This is because hosting servers is EXPENSIVE. For example, hosting a  Minecraft server on Minecraft Realms is $8 dollars per month. That’s not much, but it's only one session. If we assume that this will “only”  have a max player count of 40 people per day, that means we have to host 10 servers. That means $80 dollars per month. This doesn’t even take  into account all of the other things we might have to worry about when  developing a server-client architecture, like having to maintain two  different codebases at the same time.

As such, making a peer-to-peer game makes sense at this scale.  However, any one who wants to make an online multiplayer game MUST  understand that every multiplayer solution is specialized to your  use-case. The server-client architecture has many other benefits, mainly scalability. But also, in Fish Fight there is no persistent state or  complex systems that must always be online. The ”servers” in P2P go  offline as soon as a player quits.

The Authority is whoever decides what is true, and can overwrite  anyone else’s state. The authority isn’t necessarily one party, but  rather authority is held over singular objects or values. For example, a player might hold authority over a ball they are kicking, so they don’t have to ask the server for confirmation before interacting resulting in a snappier experience. This isn’t exactly applicable in a lockstep P2P  architecture, but is foundational for client-servers so you’ll likely  see the term if you ever read anything about networking. In those cases  the server is almost always the authority, so the players can’t cheat.

Listen Servers are a network architecture/topology not to be confused with peer-to-peer. It is simply a client-server architecture where one  of the clients hosts the server. This combines the cost benefits of a  P2P topology while also allowing for better lag compensation methods. It still requires the same bandwidth as P2P, but only for the client who  is running the server so the game is no longer capped by the slowest  connection. Fish Fight might eventually move to this topology or move to a P2P model.

Ping/Latency/Round Trip Time all relate to the time it takes for a  message to travel over a network. Latency is the time it takes to travel from the sender to the recipient, while Round Trip Time (RTT) and Ping  refers to the time it takes both back and forth. RTT however is not  necessarily twice the latency, although often very close. On the  network, different conditions and even routes means that times will  vary.

Jitter is related to latency, and is the amount by which it varies. A bad connection does not only have long latency, but the time it takes  for packets to arrive will also vary greatly. This is a major hindrance  when you want data to arrive in a smooth and regular manner, such as  frames in a video, sound in a voice chat or input for a player. This is  managed by adding a Buffer, which stores values for a short while and  then dispenses them like an even flow. The tradeoff is that the fastest  packets are slowed down to be in time with the slowest packets, leading  to an overall slowdown.

Then there’s Packet Loss, where a packet gets completely lost at some crossroad of the internet. Bad connections also means that not all  packets will arrive. This is countered by adding Redundancy. Common ways to compensate is to send packets multiple times, so at least one is  likely to arrive, or send a confirmation once a packet is received. If  the confirmation is not received, resend the packet until you get a  response.

For some slightly more low-level things that you can do without:

TCP/UDP relates to packet loss. UDP is a transport protocol on the IP stack that “simply” sends messages, fire-and-forget, with no regard to  whether it arrives. TCP on the other hand has more overhead but  guarantees that your messages arrive. FishFight uses UDP for speed, and  implements a custom redundancy layer on top of that for extra  performance. TCP is often overkill, and a custom built solution almost  always works better since it can exploit the exact workings of your  game. Overhead is the extra data that is sent every packet, and adds to  the required bandwidth. By sending more data per packet, the overhead  will make up a smaller part of the data sent.

## Delayed Lockstep

Anyways, we explained the peer-to-peer part, but you’re probably  wondering what is deterministic lockstep. Gaffer on Games already wrote  about this in [an article you can read here](https://gafferongames.com/post/deterministic_lockstep/), but in summary.

At its very basics, lockstep works by collecting input from every  player and then simulating one frame once all input is gathered. In  other words, the only thing that gets transmitted between players is  their input. Input often can be packed into a single byte, and therefore very little bandwidth is required. When the input then arrives, the  simulation is advanced one step. Since everyone waits for input from  each other, everyone steps in sync.

But however small the packets are, the latency will remain largely  the same. To wait for every player to send their input each time would  mean that the game can not update faster than (in the best case) 1/RTT/2 Hz (confirmation can be sent later). If you want your game loop to run  at 60Hz, you can’t have an RTT over 30 ms which is difficult to achieve  outside of wired and geographically small networks.

Enter: Delayed Lockstep. The ”delay” part is an input buffer that  stores inputs for a short while before executing them. Now every input  packet also contains a frame timestamp so that all remote input can be  matched up with the local input and executed at the same frame. As input rolls in it is stored in the matching slot in a buffer, and by the time a frame should be simulated the corresponding buffer slot should be  filled with input from all players. The latest input is then popped off  the buffer, which shifts one frame forward, and the game progresses one  step. By maintaining this buffer (barring major interruptions) the game  always has input at hand, and can as soon and as quickly as it wants.  But as you might guess, there’s a tradeoff, and that is of course the  added delay. The remote players already have some delay so it doesn’t  matter too much, but the local input must be slowed down to match the  slowest player, leading to slower response to keyboard input. To  minimise this delay, the buffer should be as short as possible. To give  everyone’s input time to arrive the buffer must be as long as the  slowest players ping + their jitter. To improve the experience, your  networking should continually measure how long it is between the last  player’s input arriving and the buffer running out, and then adjusting  the buffer to be as short as possible. A too long buffer means  unnecessary input delay, but if the buffer is too short and runs out the game must freeze until input arrives. It’s a fine line to walk, but  it’s usually better to lean towards too long than having interruptions.

However, there’s one big, or rather HUGE, issue: Determinism. Since  all that is sent between clients is their inputs, one requirement must  be met. Given the same world state and inputs, every time, on every  computer, OS, architecture, instruction set, etc. etc. the game must be  able to generate the exact same next frame. The tiniest deviation  accumulates and quickly makes the game unplayable. The main source of  nondeterminism is floating point arithmetic. Performing operations with  floating point numbers famously produces varying results, which depends  on many factors. Random number generators must also be kept in sync.

We want to reduce the amount of data being sent online by each peer,  AND we don’t want to have people hack the game. The way to do this then, is to have players only send their inputs to each other, so that you  can’t have people change their position all willy nilly [like this](https://www.youtube.com/watch?v=yMVkh4BJe7k).

However, we want to make sure that when inputs are sent to each  player, the game itself is fully deterministic: the same thing must  happen with the same inputs every time to prevent clients from desyncing with each other. There’s a lot of ways to handle this that we not going to get into, but I’m sure could make for a very interesting separate  article.

For now, we are going to assume that the game itself is fully deterministic and just take a look at the code itself in [src/nodes/network.rs](https://github.com/fishfight/FishFight/blob/main/src/nodes/network.rs).

We are first going to take a look at the data being sent online, the  packets. In this case, the packets in network.rs are called messages,  which is what they are so it makes sense.

```rust
#[derive(DeBin, SerBin)]
enum Message {
    Input {
        // position in the buffer
        pos: u8,
        // current simulation frame
        frame: u64,
        input: Input,
    },
    Ack {
        // last simulated frame
        frame: u64,
        /// bitmask for last 8 frames
        /// one bit - one received input in a frames_buffer
        ack: u8,
    },
}
```

Each message has two different parts to it. It has an Input struct,  which contains the position in the input buffer, the frame that the game is currently on, and the actual input. It also has an Ack struct (which is sent to assure the other clients they received their input) that  contains the frame the game is on. The Ack struct also has a bitflag  that tells what inputs they got from what clients.

Now, the person reading this might wonder: why is there an input  buffer here? Well, we forgot to mention one thing. See, in the real  world, deterministic lockstep doesn’t actually work. Well, it doesn’t work the way you might think at first.

Every packet being sent online is always going to have a little bit  of delay to it (latency) since there is going to be some distance that  is covered by the packet as it travels. Even at the speed of light, it  still takes a nanosecond for a packet to get across from one side of the room to the other.

As such, if you were to have a naive interpretation of deterministic  lockstep without accounting for latency, it would just freeze every  single frame as it waits for inputs to appear.

This image taken from [Infil’s article about the netcode in Killer Instinct](https://ki.infil.net/w02-netcode.html) (with permission!) should show what that looks like.

<iframe src='https://gfycat.com/ifr/FaithfulImmaculateBug' frameborder='0' scrolling='no' allowfullscreen width='640' height='405'></iframe>

(Note that this isn’t actually delay-based netcode either, but that’s a discussion for another time)

What instead is done is adding in an artificial delay to the program  itself, and having a buffer of input of a certain amount of frames. This allows the inputs time to arrive.

<iframe src='https://gfycat.com/ifr/ActiveFormalAtlanticsharpnosepuffer' frameborder='0' scrolling='no' allowfullscreen width='640' height='405'></iframe>

When the inputs come to the other players, it gets added into the  input buffer which also lasts a certain amount of frames. This means  that most moves and stuff feel noticeably slower with added delay.

<iframe src='https://gfycat.com/ifr/AffectionateDescriptiveAiredaleterrier' frameborder='0' scrolling='no' allowfullscreen width='640' height='405'></iframe>

- - -

This concludes part 1 of our "Netcode Explained" series. In part 2 we will do a code walk-through and piecemeal analysis.

*Written by Srayan “ValorZard” Jana and grufkok, with editing by Erlend*