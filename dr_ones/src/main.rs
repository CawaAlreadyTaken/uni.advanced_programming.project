mod client;
mod drone;
pub mod network_initializer;
mod server;
pub mod simulation_controller;
use network_initializer::NetworkInitializer;

#[macroquad::main("Graphical Window")]
async fn main() {
    let mut network_initializer_element = NetworkInitializer::new();

    network_initializer_element.start().await;
}
