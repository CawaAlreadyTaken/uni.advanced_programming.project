use std::thread;
use crossbeam_channel;
//use wg_2024::api;
//use wg_2024::types;

mod client;
mod drone;
pub mod network_initializer;
mod server;
pub mod simulation_controller;
use network_initializer::NetworkInitializer;
use simulation_controller::SimulationController;

fn main() {
    // Crossbeam channel between network initializer and simulation controller
    let (sender_network, receiver_simulation) = crossbeam_channel::bounded(5);

    let simulation_controller_element = SimulationController::new(receiver_simulation);
    let network_initializer_element = NetworkInitializer::new(sender_network);

    let simulation_controller_thread = thread::spawn(move || {
        run_simulation_controller(simulation_controller_element);
    });

    let network_initializer_thread = thread::spawn(move || {
        run_network_initializer(network_initializer_element);
    });

    match simulation_controller_thread.join() {
        Ok(_) => println!("Simulation controller thread joined successfully"),
        Err(e) => eprintln!("Error joining simulation controller thread: {:?}", e),
    }
    match network_initializer_thread.join() {
        Ok(_) => println!("Network initializer thread joined successfully"),
        Err(e) => eprintln!("Error joining network initializer thread: {:?}", e),
    }
}

fn run_simulation_controller(mut simulation_controller_element: SimulationController) {
    simulation_controller_element.start();
}

fn run_network_initializer(mut network_initializer_element: NetworkInitializer) {
    network_initializer_element.start();
}
