# Dr Ones

**Dr Ones** is a Rust-based simulation project that models a network composed of drones, clients, and servers. The project was developed following the definitions provided by our professor and simulates network connectivity, where messages are broken down into packets, transmitted through a network (with possible packet drops), and then reassembled.

## Overview

In this simulation, the system comprises:
- **Drones:** Act as intermediaries in the network, forwarding packets. Each drone has its own specific packet drop rate, meaning packets may be dropped and subsequently re-sent.
- **Clients and Servers:** These nodes initiate communication. At startup, they flood the network to discover other nodes and the overall network topology.
- **Simulation Controller:** Manages the dynamic aspects of the simulation, such as spawning or crashing drones and triggering activities on clients and servers.
- **Network Initializer:** This component kick-starts the entire network simulation.

## Features

- **Packet-Based Communication:** Messages are disassembled into packets for transmission and reassembled at the destination.
- **Dynamic Network Topology:** Clients and servers discover the network topology at startup.
- **Packet Dropping and Re-Sending:** Drones may drop packets based on individual drop rates; dropped packets are re-sent.
- **Simulation Control:** Ability to spawn or crash drones and simulate client/server behavior through a dedicated controller.

## Project Structure

- **`dr_ones/`:** The root of the project containing all modules.
- **`dr_ones/network_initializer/`:** Contains the network initializer component that starts the simulation.
- **Other Modules:** Include implementations for drones, clients, servers, and the simulation controller.

## Installation & Running

1. **Navigate to the Network Initializer Directory:**
   ```bash
   cd dr_ones/network_initializer/
   ```

2. **Run the project:**
   ```bash
   cargo run
   ```

3. **Change the topology file:**
To use a different topology file, edit the `TOPOLOGY_PATH` variable in `dr_ones/network_initializer/network_initializer.rs`

## Documentation
The project documentation has been automatically generated using `cargo doc`. You can view the documentation by opening the following file in your browser:
- code_documentation/dr_ones/index.html

## Contributors
This project was developed by:

- Federico De Santi
- Wendelin Falschlunger
- Nathan Perdoux
- Daniele Cabassi

## Contacts

- federico.desanti@studenti.unitn.it
- wendelin.falschlunger@studenti.unitn.it
- nathan.perdoux@studenti.unitn.it
- daniele.cabassi@studenti.unitn.it

