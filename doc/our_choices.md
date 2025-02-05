## Project structure
network init -> Simulation controller 
             -> Clients
             -> Servers
             -> Communication channels
             -> Drones


## Simulation controller
We have 2 receivers: one for events (explain why everything goes under DroneEvent) and one for Topology updates.

How we handle communication between network nodes and simulation controller

# Gui


## Network nodes



## Drones
I think nothing useful can go in here: we just implemented what the specifications said



## Host nodes
We use serde to serialize and deserialize every message exchanged in the network.
We only send the fragmented packet when we are sure that a path to the destination exists.

We thought that we needed to have a reliable way to update a host node's topology.
To do so, we thought that the best place to update the topology was when receiving an error_in_routing or when the adjacent drone (trough which the packet should have passed) crashed.
For this casistic we opted for 3 buffers: fragments not matched by any ack, error in routing nacks and ...
Write how we decided to handle packets/fragments when:
- error_in_routing nacks are received 
- the first drone has crashed

Flooding:
- when receiving any flood response, we update our local topology without caring about the flood id of that flood_response



## Clients



## Servers
# Communication
Clients can subscribe to communication servers and send messages to other subscribed clients in a 1 to 1 way


# Content
We decided to exchange images



## Frontend
