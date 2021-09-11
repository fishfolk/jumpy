Lightweight rendezvous server to make p2p connecitons possible throught NAT.

First client connects to a server, server assign a unique connection ID and keeps connection idling to keep hole punching works.

Other client connects to a server, receive the list of connected, but not playing clients.

When two clients successefully connects to each other, they notify the server and the server drops idling connection and remove both of them from the list.
