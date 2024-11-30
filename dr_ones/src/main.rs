use std::thread;

mod client;
mod drone;
pub mod network_initializer;
mod server;
pub mod simulation_controller;
use network_initializer::NetworkInitializer;
use wg_2024::config::Config;
use crate::network_initializer::parser;
use std::env;

#[macroquad::main("Graphical Window")]
async fn main() {
    let mut network_initializer_element = NetworkInitializer::new();

    network_initializer_element.start().await;
}
