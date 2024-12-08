use std::thread;
use crossbeam_channel::{unbounded, Receiver};
use wg_2024::drone::Drone;
use dr_ones::{client::{ClientNode, ClientOptions}, drone::Dr_One, server::{ServerNode, ServerOptions}};
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;
use wg_2024::controller::{DroneCommand,DroneEvent};
mod common;

#[test]
fn crash_test() {
    // Node identifiers
    let client_id: NodeId = 10;
    let drone1_id: NodeId = 20;
    let drone2_id: NodeId = 30;
    let drone3_id: NodeId = 40;
    let server_id: NodeId = 50;
    
    // Communication channels
    let (client_send, client_recv) = unbounded();
    let (drone1_send, drone1_recv) = unbounded();
    let (drone2_send, drone2_recv) = unbounded();
    let (drone3_send, drone3_recv) = unbounded();
    let (server_send, server_recv) = unbounded();
    
    let client_thread = thread::spawn({
        let client_recv = client_recv.clone();
        let drone1_send = drone1_send.clone();
        move || {
            let client = ClientNode::new(ClientOptions {
                id: client_id,
                controller_recv: crossbeam_channel::bounded(0).1, // simulation controller channel
                controller_send: crossbeam_channel::bounded(0).0, // simulation controller channel
                packet_recv: client_recv,
                packet_send: [(drone1_id, drone1_send)].iter().cloned().collect(),
            });
            thread::sleep(std::time::Duration::from_secs(1));
            client.run_crash_test();
        }
    });
    
    // Nodo Drone 1
    let drone1_thread = thread::spawn({
        let drone1_recv = drone1_recv.clone();
        let client_send = client_send.clone();
        let drone2_send = drone2_send.clone();
        let drone3_send = drone3_send.clone();
        move || {
            let mut drone = Dr_One::new(
                drone1_id,
                crossbeam_channel::bounded(0).0, // simulation controller channel
                crossbeam_channel::bounded(0).1, // simulation controller channel
                drone1_recv,
                [(client_id, client_send), (drone2_id, drone2_send), (drone3_id, drone3_send)]
                .iter()
                .cloned()
                .collect(),
                0.0, // PDR (probabilità di consegna)
            );
            drone.run();
        }
    });
    
    // Nodo Drone 2
    // 1. Create correctly typed channels
    let (controller_send_drone2, _) = crossbeam_channel::bounded::<DroneEvent>(0);
    let (crash_send, controller_recv_drone2) = crossbeam_channel::bounded::<DroneCommand>(0);
    
    // 2. Use in drone2 thread creation
    let drone2_thread = thread::spawn({
        let drone2_recv = drone2_recv.clone();
        let drone1_send = drone1_send.clone();
        let server_send = server_send.clone();
        move || {
            let mut drone = Dr_One::new(
                drone2_id,
                controller_send_drone2,
                controller_recv_drone2,
                drone2_recv,
                [(drone1_id, drone1_send), (server_id, server_send)]
                .iter()
                .cloned()
                .collect(),
                0.0,
            );
            drone.run_crash_test();
        }
    });
    
    // 3. Send crash command using crash_send channel
    crash_send.send(DroneCommand::Crash).unwrap();
    thread::sleep(std::time::Duration::from_millis(100));
    
    // Nodo Drone 3
    let drone3_thread = thread::spawn({
        let drone3_recv = drone3_recv.clone();
        let drone1_send = drone1_send.clone();
        let server_send = server_send.clone();
        move || {
            let mut drone = Dr_One::new(
                drone3_id,
                crossbeam_channel::bounded(0).0, // simulation controller channel
                crossbeam_channel::bounded(0).1, // simulation controller channel
                drone3_recv,
                [(drone1_id, drone1_send) , (server_id, server_send)]
                .iter()
                .cloned()
                .collect(),
                0.0, // PDR
            );
            drone.run();
        }
    });
    
    let server_thread = thread::spawn({
        let server_recv:Receiver<Packet> = server_recv.clone();
        let drone2_send = drone2_send.clone();
        let drone3_send = drone3_send.clone();
        move || {
            let mut server = ServerNode::new(ServerOptions {
                id: server_id,
                controller_send: crossbeam_channel::bounded(0).0, // simulation controller channel
                packet_recv: server_recv,
                packet_send: [(drone2_id, drone2_send),(drone3_id, drone3_send)].iter().cloned().collect(),
            });
            server.run();
        }
    });
    
    //Based on the loop nature of our components, we wait a prefixed time before finishing the test
    thread::sleep(std::time::Duration::from_secs(3));
    
    //Check the log file to make the test green or red
    let expected_logs = vec![
    "[CLIENT 10] Message fragment sent. Source routing header hops: [20, 30, 40, 50]",
    // [DRONE 30] crashed so message fragment should not be received by [SERVER 50]
    // server sending ack back
    "[DRONE 30] Starting crash sequence",
    "[DRONE 30] Processing remaining packets...",
    "No channel found for next hop: 30",
    "[DRONE 30] CRASHED.",


    ];
    
    assert!(common::check_log_file("tests/crash_test/log.txt", &expected_logs), "Log file did not contain expected entries.");
}