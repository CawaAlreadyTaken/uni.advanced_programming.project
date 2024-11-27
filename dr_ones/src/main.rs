use std::thread;

mod client;
mod drone;
pub mod network_initializer;
mod server;
pub mod simulation_controller;
use network_initializer::NetworkInitializer;

fn main() {
    let network_initializer_element = NetworkInitializer::new();

    let network_initializer_thread = thread::spawn(move || {
        run_network_initializer(network_initializer_element);
    });

    match network_initializer_thread.join() {
        Ok(_) => println!("[MAIN PROCESS] Network initializer thread joined successfully"),
        Err(e) => eprintln!("[MAIN PROCESS] Error joining network initializer thread: {:?}", e),
    }
}

fn run_network_initializer(mut network_initializer_element: NetworkInitializer) {
    network_initializer_element.start();
}
