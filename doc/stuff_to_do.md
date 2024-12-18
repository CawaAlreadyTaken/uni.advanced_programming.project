## Stuff still to do

Coding:
- [ ] Refactor
    - [ ] Rename NetworkUtils into NetworkNode and get all functions which are common to drone, client and server inside it. This should live inside a common crate, which should be called network_node. This crate lives inside a public repository
    - [ ] Introduce HostNode and get all functions which are common to client and server inside it. This should live inside a common crate, which should be called host_node. This crate lives inside a private repository.
- [ ] New features
    - [ ] Communication between the simulation controller and the nodes.
        - [X] Crash, SetPacketDropRate (Simulation Controller -> Drone)
        - [ ] AddSender, RemoveSender (Move to NetworkNode trait)
            - [X] (Simulation Controller -> Drone)
            - [ ] (Simulation Controller -> Client)
            - [ ] (Simulation Controller -> Server)
        - [ ] PacketSent (Drone, Client, Server -> Simulation Controller) (Move to NetworkNode trait)
            - [X] (Drone -> Simulation Controller)
            - [ ] (Client -> Simulation Controller)
            - [ ] (Server -> Simulation Controller)
        - [X] PacketDropped (Drone -> Simulation Controller)
        - [ ] ControllerShortcut (Drone -> Simulation Controller)
        - [ ] HostShortcut (Simulation Controller -> Client, Server)
    - [ ] GUI of simulation controller
    - [ ] Assembler of packet fragments
    - [ ] Use simulation controller and network initializer in tests
    - [ ] Add toml files for all topologies defined in the document
