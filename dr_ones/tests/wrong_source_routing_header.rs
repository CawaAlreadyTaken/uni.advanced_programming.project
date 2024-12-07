use std::thread;
use crossbeam_channel::unbounded;
use wg_2024::drone::Drone;
use dr_ones::{client::{ClientNode, ClientOptions}, drone::Dr_One, server::{ServerNode, ServerOptions}};
use wg_2024::network::NodeId;
use std::fs;
mod common;

#[test]
fn test_wrong_source_routing_header() {
    // Identificatori per i nodi
    let client_id: NodeId = 1;
    let drone1_id: NodeId = 2;
    let drone2_id: NodeId = 3;
    // let server_id: NodeId = 4;//todo: remove

    // Canali di comunicazione per i pacchetti
    let (client_send, client_recv) = unbounded();
    let (drone1_send, drone1_recv) = unbounded();
    let (drone2_send, drone2_recv) = unbounded();
    // let (server_send, server_recv) = unbounded(); //todo: remove

    // Nodo Client
    let client_thread = thread::spawn({
        let client_recv = client_recv.clone();
        let drone1_send = drone1_send.clone();
        move || {
            let mut client = ClientNode::new(ClientOptions {
                id: client_id,
                controller_recv: crossbeam_channel::bounded(0).1, // simulation controller channel
                controller_send: crossbeam_channel::bounded(0).0, // simulation controller channel
                packet_recv: client_recv,
                packet_send: [(drone1_id, drone1_send)].iter().cloned().collect(),
            });
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

    // TODO: we don't use this node because the packet will never reach him
    // Nodo Server
    // let server_thread = thread::spawn({
    //     let server_recv = server_recv.clone();
    //     let drone2_send = drone2_send.clone();
    //     move || {
    //         let mut server = ServerNode::new(ServerOptions {
    //             id: server_id,
    //             controller_send: crossbeam_channel::bounded(0).0, // simulation controller channel
    //             packet_recv: server_recv,
    //             packet_send: [(drone2_id, drone2_send)].iter().cloned().collect(),
    //         });
    //         server.run();
    //     }
    // });

    //Based on the loop nature of our components, we wait a prefixed time before finishing the test
    thread::sleep(std::time::Duration::from_secs(3));

    //Check the log file to make the test green or red
    let expected_logs = vec![
        "[CLIENT 1] Message fragment sent. Source routing header hops: [1, 2, 4]",
        "[CLIENT 1] Nack received. Source routing header hops: [2, 1]"
    ];

    assert!(common::check_log_file("tests/wrong_source_routing_header/log.txt", &expected_logs), "Log file did not contain expected entries.");
}

