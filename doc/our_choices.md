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

When we receive a nack, we compute the path again to resend the fragment.
Specifically, when we receive an error_in_routing we initiate a new flood request.

To handle the error_in_routing case we opted for 3 buffers: 
- fragments not matched by any ack (every fragment will be inserted here and eventually removed)
- error in routing nacks (we put here every fragment from the first buffer for which we receive an error_in_routing. We will resend everything that is in here after we rebuild a new path)
- not yet fragmented information (because the first neighbor drone has crashed). 
Example: intended path 1(client) -> 2(drone) -> 3(server) 
if 1 notices that 2 has crashed, the message that we wanted to forward is put in this last buffer waiting for a new path to be built (we can construct it because we initiate a new flood request)



# Flooding
- when receiving any flood response, we update our local topology without caring about the flood id of that flood_response

# Routing
TODO: put the new logic for the bfs in here


# Fragmentation / Defragmentation
We used the serde library to handle this.
The aim of this is transforming high-level messages into packets (fragments).

Every high level message is a SerializableMessage enum's type.
A key funciton is the build_and_forward_serializable_packets that is responsible of fragmenting the message and forwarding each little piece.


## Clients



## Servers
# Communication
Clients can subscribe to communication servers and send messages to other subscribed clients in a 1 to 1 way


# Content
We decided to exchange images. Those images are stored in network_initializer -> content folder.

Simulation controller commands:
- GetFileList
- GetFile
When the simulation controller receives these commands through the CLI, it sends an ad hoc command to the client that will proceed by sending into the network the appropriate message with the appropriate destination server.



## Frontend
Every frontend is linked to a client and displays its perspective

How is the communication handled between the frontend and clients