//! Main entry point for the drone network simulator.
//! This module initializes the network and starts the graphical interface.

mod client;
mod drone;
mod network_initializer;
mod server;
mod simulation_controller;
mod utils;

use network_initializer::NetworkInitializer;

/// Program entry point.
/// Initializes the network simulator with a graphical window using macroquad.
#[macroquad::main("Drone Network Simulator")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the network
    let mut network = NetworkInitializer::new();

    // Start the network simulation
    network.start().await?;

    Ok(())
}
