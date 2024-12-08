//! Command Line Interface module for the simulation controller.
//! Provides interactive command-line control of the network simulation.

use crate::simulation_controller::SimulationController;
use std::io::{self, Write};
use wg_2024::network::NodeId;

/// Represents possible CLI commands with their parameters
#[derive(Debug)]
enum CliCommand {
    Help,
    Crash(NodeId),
    Spawn(Vec<NodeId>),
    Exit,
    Unknown(String),
}

pub mod cli {
    use super::*;

    /// Starts the CLI interface for the simulation controller
    pub fn run_cli(simulation_controller: &mut SimulationController) {
        println!("[SIM CONTR] CLI interface started. Type 'help' for available commands.");

        let mut input = String::new();

        loop {
            input.clear();
            if !prompt_command(&mut input) {
                println!("[SIM CONTR] Failed to read command. Try again.");
                continue;
            }

            let command = parse_command(&input);
            if !handle_command(command, simulation_controller) {
                break;
            }
        }
    }

    /// Displays the command prompt and reads user input
    fn prompt_command(input: &mut String) -> bool {
        print!("[SIM CONTR] > ");
        if io::stdout().flush().is_err() {
            return false;
        }

        match io::stdin().read_line(input) {
            Ok(_) => !input.trim().is_empty(),
            Err(_) => false,
        }
    }

    /// Parses a command string into a CliCommand enum
    fn parse_command(input: &str) -> CliCommand {
        let input = input.trim();

        match input {
            "help" => CliCommand::Help,
            "exit" => CliCommand::Exit,
            cmd if cmd.starts_with("crash ") => parse_crash_command(cmd),
            cmd if cmd.starts_with("spawn ") => parse_spawn_command(cmd),
            _ => CliCommand::Unknown(input.to_string()),
        }
    }

    /// Parses a crash command with its node ID parameter
    fn parse_crash_command(cmd: &str) -> CliCommand {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.len() != 2 {
            return CliCommand::Unknown(cmd.to_string());
        }

        match parts[1].parse::<NodeId>() {
            Ok(node_id) => CliCommand::Crash(node_id),
            Err(_) => CliCommand::Unknown(cmd.to_string()),
        }
    }

    /// Parses a spawn command with its list of connected node IDs
    fn parse_spawn_command(cmd: &str) -> CliCommand {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.len() != 2 {
            return CliCommand::Unknown(cmd.to_string());
        }

        let node_ids: Result<Vec<NodeId>, _> =
            parts[1].split(',').map(|id| id.parse::<NodeId>()).collect();

        match node_ids {
            Ok(ids) => CliCommand::Spawn(ids),
            Err(_) => CliCommand::Unknown(cmd.to_string()),
        }
    }

    /// Handles a parsed command by executing the appropriate controller action
    fn handle_command(command: CliCommand, controller: &mut SimulationController) -> bool {
        match command {
            CliCommand::Help => {
                display_help();
                true
            }
            CliCommand::Crash(node_id) => {
                controller.make_crash(node_id);
                true
            }
            CliCommand::Spawn(node_ids) => {
                match controller.spawn_node(node_ids) {
                    Ok(_) => println!("[SIM CONTR] Node spawned successfully"),
                    Err(e) => println!("[SIM CONTR] Failed to spawn node: {}", e),
                }
                true
            }
            CliCommand::Exit => {
                println!("[SIM CONTR] Exiting simulation...");
                controller.exit();
                false
            }
            CliCommand::Unknown(cmd) => {
                println!(
                    "[SIM CONTR] Unknown command: '{}'. Type 'help' for available commands.",
                    cmd
                );
                true
            }
        }
    }

    /// Displays the help message with available commands
    fn display_help() {
        println!("[SIM CONTR] Available commands:");
        println!(
            "[SIM CONTR]   help                                        - Show this help message"
        );
        println!(
            "[SIM CONTR]   crash <node_id>                            - Simulate crashing a node"
        );
        println!("[SIM CONTR]   spawn <comma_separated_connected_node_ids>  - Spawn a new drone, with the given connections");
        println!("[SIM CONTR]   exit                                        - Exit the simulation");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO: Add tests for the CLI module
}
