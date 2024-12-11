use crossbeam_channel::unbounded;
use dr_ones::{client::ClientNode, drone::Dr_One, server::ServerNode};
use std::thread;
use wg_2024::drone::Drone;
use wg_2024::network::NodeId;
mod common;

#[test]
fn test_wrong_source_routing_header() {
    // Node identifiers
    let client_id: NodeId = 10;
    let drone1_id: NodeId = 20;
    let drone2_id: NodeId = 30;

    // Communication channels
    let (client_send, client_recv) = unbounded();
    let (drone1_send, drone1_recv) = unbounded();
    let (drone2_send, drone2_recv) = unbounded();

    // Client node
    let client_thread = thread::spawn({
        let client_recv = client_recv.clone();
        let drone1_send = drone1_send.clone();
        move || {
            let mut client = ClientNode::new(
                client_id,
                crossbeam_channel::bounded(0).0, // simulation controller channel
                crossbeam_channel::bounded(0).1, // simulation controller channel
                client_recv,
                [(drone1_id, drone1_send)].iter().cloned().collect(),
            );
            client.run_test_wrong_source_routing_header();
        }
    });

    // Nodo Drone 1
    let drone1_thread = thread::spawn({
        let drone1_recv = drone1_recv.clone();
        let client_send = client_send.clone();
        let drone2_send = drone2_send.clone();
        move || {
            let mut drone = Dr_One::new(
                drone1_id,
                crossbeam_channel::bounded(0).0, // simulation controller channel
                crossbeam_channel::bounded(0).1, // simulation controller channel
                drone1_recv,
                [(client_id, client_send), (drone2_id, drone2_send)]
                    .iter()
                    .cloned()
                    .collect(),
                0.0, // PDR (probabilità di consegna)
            );
            drone.run();
        }
    });

    // Nodo Drone 2
    let drone2_thread = thread::spawn({
        let drone2_recv = drone2_recv.clone();
        let drone1_send = drone1_send.clone();
        // let server_send = server_send.clone();
        move || {
            let mut drone = Dr_One::new(
                drone2_id,
                crossbeam_channel::bounded(0).0, // simulation controller channel
                crossbeam_channel::bounded(0).1, // simulation controller channel
                drone2_recv,
                [(drone1_id, drone1_send) /*, (server_id, server_send)*/]
                    .iter()
                    .cloned()
                    .collect(),
                0.0, // PDR
            );
            drone.run();
        }
    });

    //Based on the loop nature of our components, we wait a prefixed time before finishing the test
    thread::sleep(std::time::Duration::from_secs(3));

    //Check the log file to make the test green or red
    let expected_logs = vec![
        "[CLIENT 10] Message fragment sent. Source routing header hops: [10, 20, 30, 40]",
        "[CLIENT 10] Nack->ErrorInRouting(40) received. Source routing header hops: [30, 20, 10]",
    ];

    assert!(
        common::check_log_file("tests/wrong_source_routing_header/log.txt", &expected_logs),
        "Log file did not contain expected entries."
    );
}
