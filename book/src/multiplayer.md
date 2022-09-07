# Multiplayer

Connection process in the FishFight is extremely low-level right now.  
We do not have a matchmaking server and we want to collect data and network setups and possible problems before doing the process fully automatic.

![image](https://user-images.githubusercontent.com/910977/133952684-19e7f10a-ed80-49e6-8a31-205f28a78c39.png)

The only supported game mode right now: 1v1.

## Connection Types

### LAN

When both computers are connected to the same router: are on the same wifi, home network etc.

Send your local IP from "Self addr" field to your opponent, click "Probe connection" and if connection can be established - click "Connect".

### STUN

When each player is under a NAT. Usually when its two players over internet with different network providers.
This option may work, but may not, it depends on the type of nat, router configs etc.

Idea is exactly the same as with LAN: copy-paste your own "Self addr" over discord, probe connection, if it works - click "Connect"

### Relay

When both players are on remote computers over internet, but STUN connection did not work and router reconfiguration is not an option - there is a relay server available.

Relay server will introduce additional LAG - each packet will be forwarded through a server.

Connection idea is still the same, but instead of IP "Self addr" will be an ID on the relay server. Copy-paste it over internet, set "Opponent addr", push "Connect"

## Router Configuration

When STUN server failed, but Relay is too slow - there is a way to improve gameplay experience. Go to router settings and forward ports 3400, 3401, 3402, 3403 to computer with the FishFight.

It still may depend on the internet provider, maybe the router itself is behind some global provider NAT or something.

TODO: Make a better "Router configuration" section here.
