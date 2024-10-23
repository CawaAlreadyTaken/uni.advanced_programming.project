use std::thread;

mod client;
mod server;
pub mod network_initializer;
pub mod simulation_controller;

fn main() {
    let simulation_controller_thread = thread::spawn(move || {
        simulation_controller::start_simulation_controller();
    });

    let network_initializer_thread = thread::spawn(move || {
        network_initializer::start_network_initializer();
    });

    simulation_controller_thread.join().unwrap();
    network_initializer_thread.join().unwrap();
}
