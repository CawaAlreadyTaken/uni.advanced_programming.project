use std::thread;
use crossbeam_channel::unbounded;
use wg_2024::drone::Drone;
use dr_ones::{client::{ClientNode, ClientOptions}, drone::Dr_One, server::{ServerNode, ServerOptions}};
use wg_2024::network::NodeId;
use std::fs;
mod common;

#[test]
fn test_fragment_forward() {
    // Node identifiers
    let client1_id: NodeId = 10;
    let drone_id: NodeId = 20;
    let client2_id: NodeId = 30;

    // Communication channels
    let (client1_send, client1_recv) = unbounded();
    let (drone_send, drone_recv) = unbounded();
    let (client2_send, client2_recv) = unbounded();

    // Client1 node
    let client1_thread = thread::spawn({
        let client1_recv = client1_recv.clone();
        let drone_send = drone_send.clone();
        move || {
            let mut client1 = ClientNode::new(ClientOptions {
                id: client1_id,
                controller_recv: crossbeam_channel::bounded(0).1, // simulation controller channel
                controller_send: crossbeam_channel::bounded(0).0, // simulation controller channel
                packet_recv: client1_recv,
                packet_send: [(drone_id, drone_send)].iter().cloned().collect(),
            });
            client1.run_test_fragment_forward();
        }
    });

    // Drone node
    let drone_thread = thread::spawn({
        let drone_recv = drone_recv.clone();
        let client1_send = client1_send.clone();
        let client2_send = client2_send.clone();
        move || {
            let mut drone = Dr_One::new(
                drone_id,
                crossbeam_channel::bounded(0).0, // simulation controller channel
                crossbeam_channel::bounded(0).1, // simulation controller channel
                drone_recv,
                [(client1_id, client1_send), (client2_id, client2_send)]
                    .iter()
                    .cloned()
                    .collect(),
                0.0,
            );
            drone.run();
        }
    });

    // Client2 node
    let client2_thread = thread::spawn({
        let client2_recv = client2_recv.clone();
        let drone_send = drone_send.clone();
        move || {
            let mut client2 = ClientNode::new(ClientOptions {
                id: client2_id,
                controller_recv: crossbeam_channel::bounded(0).1, // simulation controller channel
                controller_send: crossbeam_channel::bounded(0).0, // simulation controller channel
                packet_recv: client2_recv,
                packet_send: [(drone_id, drone_send)].iter().cloned().collect(),
            });
            client2.run_test_fragment_forward();
        }
    });

    //Based on the loop nature of our components, we wait a prefixed time before finishing the test
    thread::sleep(std::time::Duration::from_secs(3));

    //Check the log file to make the test green or red
    let expected_logs = vec![
        "",
        "",
    ];

    assert!(common::check_log_file("tests/fragment_forward/log.txt", &expected_logs), "Log file did not contain expected entries.");

}