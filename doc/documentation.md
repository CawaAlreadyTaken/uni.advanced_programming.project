# Drone Documentation: `Dr_One`
> <em>"Where PDR (Packet Drop Rate) means Potential Drone Rage"</em>

This documentation details the implementation and functionality of the `Dr_One` drone within the simulation environment. The drone allows communication and networking between clients and servers using a simulated network managed by a controller.

## Overview
The `Dr_One` drone serves as a node in the simulated network, capable of receiving and transmitting packets, handling routing, broadcasting flood requests, and maintaining communication with a simulation controller. The implementation uses:

- **Crossbeam channels** for inter-node communication.
- **IndexMap** for managing unique flood IDs.
- A **Random generator** to simulate packet drop rates (PDR).

## Key Components

### Struct Definition

```rust
pub struct Dr_One {
    id: NodeId,
    sim_contr_send: Sender<DroneEvent>,
    sim_contr_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    pdr: f32,
    seen_flood_ids: IndexSet<u64>,
    random_generator: ThreadRng,
}
```

#### Fields:
- **`id`**: Unique identifier for the drone.
- **`sim_contr_send`**: Sender channel to communicate events to the simulation controller.
- **`sim_contr_recv`**: Receiver channel to receive commands from the simulation controller.
- **`packet_recv`**: Receiver channel for incoming packets.
- **`packet_send`**: Map of neighboring node IDs to their respective sender channels.
- **`pdr`**: Packet drop rate used to simulate unreliable communication.
- **`seen_flood_ids`**: Set of flood request IDs to avoid processing duplicate requests.
- **`random_generator`**: Thread-local random number generator for PDR simulation.

### Traits Implemented

#### `NetworkUtils`
Provides utility methods for network operations:
- **`get_id`**: Returns the drone's ID.
- **`get_packet_senders`**: Accesses the list of packet senders.
- **`get_random_generator`**: Provides access to the random number generator.

#### `Drone`
Defines the drone's lifecycle and interactions:
- **`new`**: Initializes a new drone instance.
- **`run`**: Starts the main event loop for the drone.

## Event Loop

The main event loop in `run_internal` handles two primary tasks:
1. Receiving and processing packets.
2. Responding to commands from the simulation controller.

### Packet Handling
Packets can be:
- **Flood Requests**: Broadcast to all neighbors or responded to directly.
- **Routed Packets**: Forwarded based on the routing header, or a `Nack` is sent if routing fails.

#### Example Handling:
- **Flood Request**:
  - Checks if the flood request has been processed.
  - If unprocessed and neighbors are available, broadcasts the request.
  - If already processed or no neighbors exist, sends a `FloodResponse`.

- **Routed Packet**:
  - Verifies if the drone is the intended recipient.
  - If the routing fails, a `Nack` is sent.
  - If the packet reaches the destination, a `Nack` is returned.

### Controller Commands
The drone responds to the following commands:
- **`AddSender`**: Adds a new neighbor node.
- **`RemoveSender`**: Removes a neighbor node.
- **`SetPacketDropRate`**: Updates the packet drop rate.
- **`Crash`**: Simulates the drone's crash.

## Core Methods

### `handle_flood_request`
Processes incoming flood requests:
- Adds the drone to the `path_trace`.
- Checks if the flood request has already been processed.
- Either broadcasts the request or sends a flood response.

### `handle_routed_packet`
Handles packets with explicit routing headers:
- Verifies routing correctness.
- Determines whether the packet is dropped based on PDR.
- Forwards the packet to the next hop or sends a `Nack` if routing fails.

### `build_nack`
Constructs a `Nack` packet to notify errors during routing or packet handling.

### `broadcast_packet`
Broadcasts a packet to all neighbors except the sender of the original request.

### `reverse_packet_routing_direction`
Reverses the routing path for a packet, enabling it to be sent back to the origin.

## Debugging Notes
- Print statements are present in various methods to aid debugging.
- Example debugging points include:
  - Routing failures.
  - Flood request handling.

## Extensibility
The current implementation can be extended to include:
- Enhanced logging for detailed insights.
- Support for additional packet types.
- Dynamic adjustments to PDR based on network conditions.

## Usage

1. **Initialization**:
   ```rust
   let drone = Dr_One::new(
       id,
       controller_send,
       controller_recv,
       packet_recv,
       packet_send,
       0.1, // Example PDR
   );
   ```

2. **Run Event Loop**:
   ```rust
   drone.run();
   ```

3. **Add/Remove Neighbors**:
   ```rust
   drone.add_channel(neighbor_id, sender);
   drone.remove_channel(neighbor_id);
   ```

## Known Limitations
- Assumes a static network topology during initialization.
- Debugging outputs may require refinement for production use.

## Conclusion
The `Dr_One` drone implementation provides a robust framework for simulating network communication in Rust. If needed, please contact us on telegram:
- @falswe, Wendelin (Group leader)
- @Nathinus, Nathinus (the boss)
- @Destewie, Des (the head)
- @IAmCawa, Cawa (the arm)
