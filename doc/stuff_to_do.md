## Stuff still to do before the fair

Coding:
- [ ] Refactor client,server,drone code + comments (wendelin)
- [X] Implement the logic for a client to handle the network discovery protocol.
- [ ] Implement logic for drones to handle messages from simulation controller (crash is missing) (nathan)
- [ ] Implement Acks for Clients and Servers (nathan)

  Tests to be done from the repo:
- [ ] Forward a fragment (pdr = 0) (wendelin)
- [ ] Forward a fragment between two drones (src: client dest:server) and see if acks get back to the source (pdrs = 0) (wendelin)
- [ ] Drop a fragment -> Drone receives fragment, drops it and sends back a nack (pdr = 1) (wendelin)
- [ ] 2 drones connected. The second one drops the fragment and sends back a nack (pdr = 0, pdr=1) (wendelin)
  Other tests:
- [X] From a client, send a generic packet with a wrong source routing header and see if the drone handles it sending back a nack (federico)
- [ ] Crashed drone (nathan)
- [ ] Test for the flooding discovery (federico)

Others:

- [ ] Quick presentation for selling the drone.  
- [ ] Create a quick ".md" file for the documentation of the drone.  
- [ ] Think about what to answer when people ask "do you provide customer support? How?".  
- [ ] Remember to bring the 3d printed drone.  

