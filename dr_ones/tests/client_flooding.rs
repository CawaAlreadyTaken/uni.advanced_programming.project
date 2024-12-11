use crossbeam_channel::unbounded;
use dr_ones::{client::ClientNode, drone::Dr_One, server::ServerNode};
use std::thread;
use wg_2024::drone::Drone;
use wg_2024::network::NodeId;
mod common;

#[test]
fn client_flooding() {
    // Node identifiers
    let client_id: NodeId = 1;
    let drone2_id: NodeId = 2;
    let drone3_id: NodeId = 3;
    let drone4_id: NodeId = 4;
    let drone5_id: NodeId = 5;
    let drone6_id: NodeId = 6;
    let server_id: NodeId = 7;

    // Communication channels
    let (client_send, client_recv) = unbounded();
    let (drone2_send, drone2_recv) = unbounded();
    let (drone3_send, drone3_recv) = unbounded();
    let (drone4_send, drone4_recv) = unbounded();
    let (drone5_send, drone5_recv) = unbounded();
    let (drone6_send, drone6_recv) = unbounded();
    let (server_send, server_recv) = unbounded();

    // Client Node
    let client_thread = thread::spawn({
        let client_recv = client_recv.clone();
        let drone2_send = drone2_send.clone();
        let drone4_send = drone4_send.clone();
        move || {
            let mut client = ClientNode::new(
                client_id,
                crossbeam_channel::bounded(0).0, // simulation controller channel
                crossbeam_channel::bounded(0).1, // simulation controller channel
                client_recv,
                [(drone2_id, drone2_send), (drone4_id, drone4_send)]
                    .iter()
                    .cloned()
                    .collect(),
            );
            client.run_client_flooding_test();
        }
    });

    // Drone 2 Node
    let drone2_thread = thread::spawn({
        let drone2_recv = drone2_recv.clone();
        let client_send = client_send.clone();
        let drone3_send = drone3_send.clone();
        let drone6_send = drone6_send.clone();
        move || {
            let mut drone = Dr_One::new(
                drone2_id,
                crossbeam_channel::bounded(0).0, // simulation controller channel
                crossbeam_channel::bounded(0).1, // simulation controller channel
                drone2_recv,
                [
                    (client_id, client_send),
                    (drone3_id, drone3_send),
                    (drone6_id, drone6_send),
                ]
                .iter()
                .cloned()
                .collect(),
                0.0, // PDR (probabilità di consegna)
            );
            drone.run();
        }
    });

    // Drone 3 Node
    let drone3_thread = thread::spawn({
        let drone3_recv = drone3_recv.clone();
        let server_send = server_send.clone();
        let drone2_send = drone2_send.clone();
        move || {
            let mut drone = Dr_One::new(
                drone3_id,
                crossbeam_channel::bounded(0).0, // simulation controller channel
                crossbeam_channel::bounded(0).1, // simulation controller channel
                drone3_recv,
                [(server_id, server_send), (drone2_id, drone2_send)]
                    .iter()
                    .cloned()
                    .collect(),
                0.0, // PDR (probabilità di consegna)
            );
            drone.run();
        }
    });

    // Drone 4 Node
    let drone4_thread = thread::spawn({
        let drone4_recv = drone4_recv.clone();
        let client_send = client_send.clone();
        let drone5_send = drone5_send.clone();
        let drone6_send = drone6_send.clone();
        move || {
            let mut drone = Dr_One::new(
                drone4_id,
                crossbeam_channel::bounded(0).0, // simulation controller channel
                crossbeam_channel::bounded(0).1, // simulation controller channel
                drone4_recv,
                [
                    (client_id, client_send),
                    (drone5_id, drone5_send),
                    (drone6_id, drone6_send),
                ]
                .iter()
                .cloned()
                .collect(),
                0.0, // PDR (probabilità di consegna)
            );
            drone.run();
        }
    });

    // Drone 5 Node
    let drone5_thread = thread::spawn({
        let drone5_recv = drone5_recv.clone();
        let server_send = server_send.clone();
        let drone4_send = drone4_send.clone();
        move || {
            let mut drone = Dr_One::new(
                drone5_id,
                crossbeam_channel::bounded(0).0, // simulation controller channel
                crossbeam_channel::bounded(0).1, // simulation controller channel
                drone5_recv,
                [(server_id, server_send), (drone4_id, drone4_send)]
                    .iter()
                    .cloned()
                    .collect(),
                0.0, // PDR (probabilità di consegna)
            );
            drone.run();
        }
    });

    // Drone 6 Node
    let drone6_thread = thread::spawn({
        let drone6_recv = drone6_recv.clone();
        let drone2_send = drone2_send.clone();
        let drone4_send = drone4_send.clone();
        move || {
            let mut drone = Dr_One::new(
                drone6_id,
                crossbeam_channel::bounded(0).0, // simulation controller channel
                crossbeam_channel::bounded(0).1, // simulation controller channel
                drone6_recv,
                [(drone2_id, drone2_send), (drone4_id, drone4_send)]
                    .iter()
                    .cloned()
                    .collect(),
                0.0, // PDR (probabilità di consegna)
            );
            drone.run();
        }
    });

    // Server Node
    let server_thread = thread::spawn({
        let server_recv = server_recv.clone();
        let drone3_send = drone3_send.clone();
        let drone5_send = drone5_send.clone();
        move || {
            let mut server = ServerNode::new(
                server_id,
                crossbeam_channel::bounded(0).0, // simulation controller channel
                server_recv,
                [(drone3_id, drone3_send), (drone5_id, drone5_send)]
                    .iter()
                    .cloned()
                    .collect(),
            );
            server.run_client_flooding_test();
        }
    });

    //Based on the loop nature of our components, we wait a prefixed time before finishing the test
    thread::sleep(std::time::Duration::from_secs(3));

    //Check the log file to make the test green or red
    let expected_logs = vec![
        "--------------------------------------",
        "NODE 1 TOPOLOGY after message with sess_id:0",
        "---------------",
        "CLIENTS",
        "1 -> [2, 4]",
        "---------------",
        "DRONES",
        "2 -> [1, 3, 6]",
        "3 -> [2, 7]",
        "4 -> [1, 5, 6]",
        "5 -> [4, 7]",
        "6 -> [2, 4]",
        "---------------",
        "SERVERS",
        "7 -> [3, 5]",
        "--------------------------------------",
    ];

    assert!(
        common::check_log_file("tests/client_flooding/log.txt", &expected_logs),
        "Log file did not contain expected entries."
    );
}
