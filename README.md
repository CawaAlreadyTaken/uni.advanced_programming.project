# Dr Ones

This repo contains a Rust-based simulation project that models a network composed of drones, clients, and servers. The project was developed following the definitions provided by our professor and simulates network connectivity, where messages are broken down into packets, transmitted through a network (with possible packet drops), and then reassembled.

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

```
├── README.md
├── code_documentation 					# Folder for "cargo doc" of the project
├── doc
│   ├── AP-project-main.pdf
│   ├── AP-project-org.pdf
│   ├── AP-protocol-topologies.pdf
│   ├── AP-protocol.pdf
│   ├── documentation.md
│   ├── our_choices.md
└── dr_ones				
    ├── Cargo.lock
    ├── Cargo.toml
    ├── client						# Code for clients
    │   ├── Cargo.toml
    │   └── src
    │       ├── client.rs
    │       ├── lib.rs
    │       └── web_server.rs
    ├── drone						# The code for our implementation of the drone
    │   ├── Cargo.lock
    │   ├── Cargo.toml
    │   ├── src
    │   │   ├── drone.rs
    │   │   └── lib.rs
    │   └── tests
    ├── host_node					# Code for HostNode trait
    │   ├── Cargo.lock
    │   ├── Cargo.toml
    │   └── src
    │       ├── host_node.rs
    │       └── lib.rs
    ├── network_initializer				# Code for the Network Initializer
    │   ├── Cargo.lock
    │   ├── Cargo.toml
    │   ├── content
    │   ├── src
    │   │   ├── bin
    │   │   │   └── main.rs				# main of the project
    │   │   ├── lib.rs
    │   │   └── network_initializer.rs
    │   ├── tests
    │   │   ├── ack_sent_back.rs
    │   │   ├── client_flooding.rs
    │   │   ├── fragment_forward.rs
    │   │   ├── integration_tests.rs
    │   │   ├── rusty_tester.rs
    │   │   └── wrong_source_routing_header.rs
    │   └── topologies					# Definition of the different topologies
    │       ├── standard
    │       │   ├── butterfly.toml
    │       │   ├── double_chain.toml
    │       │   ├── star.toml
    │       │   ├── tree.toml
    │       │   ├── two_stars_subnet.toml
    │       │   └── two_triangles_one_square_subnet.toml
    │       └── test
    │           ├── ack_sent_back.toml
    │           ├── client_flooding.toml
    │           ├── fragment_forward.toml
    │           └── wrong_source_routing_header.toml
    ├── network_node					# Code for the NetworkNode trait
    │   ├── Cargo.lock
    │   ├── Cargo.toml
    │   └── src
    │       ├── lib.rs
    │       ├── logging.rs
    │       └── network_node.rs
    ├── server						# Code for servers
    │   ├── Cargo.lock
    │   ├── Cargo.toml
    │   └── src
    │       ├── lib.rs
    │       └── server.rs
    ├── simulation_controller				# Code for the Simulation Controller
    │   ├── Cargo.lock
    │   ├── Cargo.toml
    │   └── src
    │       ├── cli.rs
    │       ├── gui.rs
    │       ├── lib.rs
    │       ├── parser.rs
    │       └── simulation_controller.rs
    └── test-crates.sh
```

## Installation & Running

0. **Be sure to clone the repo recursively (--recursive)**

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
- w.falschlunger@studenti.unitn.it
- nathan.perdoux@studenti.unitn.it
- daniele.cabassi@studenti.unitn.it

