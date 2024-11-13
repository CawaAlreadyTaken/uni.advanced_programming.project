use std::thread;
use tokio::sync::mpsc;
use wgl_repo_2024::api;
use wgl_repo_2024::types;

mod client;
mod drone;
pub mod network_initializer;
mod server;
pub mod simulation_controller;
use network_initializer::NetworkInitializer;
use simulation_controller::SimulationController;

fn main() {
    //tokio from network initializer to simulation controller
    let (sender_network, mut receiver_simulation) = mpsc::channel(5);

    let mut simulation_controller_element = SimulationController::new(receiver_simulation);
    let mut network_initializer_element = NetworkInitializer::new(sender_network);

    let simulation_controller_thread = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(run_simulation_controller(simulation_controller_element));
    });

    let network_initializer_thread = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(run_network_initializer(network_initializer_element));
    });

    simulation_controller_thread.join().unwrap();
    network_initializer_thread.join().unwrap();
}

async fn run_simulation_controller(mut simulation_controller_element: SimulationController) {
    simulation_controller_element.start().await;
}

async fn run_network_initializer(mut network_initializer_element: NetworkInitializer) {
    network_initializer_element.start().await;
}
