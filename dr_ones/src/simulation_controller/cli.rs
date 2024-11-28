pub mod cli {

    use std::io::{self, Write};

    use wg_2024::network::NodeId;
    use wg_2024::packet::NodeType;

    use crate::simulation_controller::SimulationController;

    pub fn run_cli(simulation_controller: &mut SimulationController) {

        let mut input = String::new();

        loop {
                print!("[SIM CONTR] > ");
                io::stdout().flush().unwrap();
                input.clear();
                if let Ok(_) = io::stdin().read_line(&mut input) {
                    let command = input.trim();
                    if command.is_empty() {
                        continue;
                    }

                    match command {
                        "help" => {
                            println!("[SIM CONTR] Available commands:");
                            println!("[SIM CONTR]   help              - Show this help message");
                            println!("[SIM CONTR]   crash <node_id>   - Simulate crashing a node");
                            println!("[SIM CONTR]   spawn <id> <type> - Spawn a new node with ID and type");
                            println!("[SIM CONTR]   exit              - Exit the simulation");
                        }
                        cmd if cmd.starts_with("crash ") => {
                            let parts: Vec<&str> = cmd.split_whitespace().collect();
                            if parts.len() == 2 {
                                let node_id = parts[1];
                                simulation_controller.make_crash(node_id.parse::<NodeId>().unwrap());
                            } else {
                                println!("[SIM CONTR] Usage: crash <node_id>");
                            }
                        }
                        cmd if cmd.starts_with("spawn ") => {
                            let parts: Vec<&str> = cmd.split_whitespace().collect();
                            if parts.len() == 3 {
                                // parse it as NodeId
                                let node_id = parts[1].parse::<NodeId>().unwrap();
                                let node_type = match parts[2] {
                                    "drone" => NodeType::Drone,
                                    "server" => NodeType::Server,
                                    "client" => NodeType::Client,
                                    _ => {
                                        println!("[SIM CONTR] Unknown node type: {}", parts[2]);
                                        continue;
                                    }
                                };
                                simulation_controller.spawn_node(node_id, node_type);
                            } else {
                                println!("[SIM CONTR] Usage: spawn <id> <type>");
                            }
                        }
                        "exit" => {
                            println!("[SIM CONTR] Exiting simulation...");
                            simulation_controller.exit();
                            break;
                        }
                        _ => {
                            println!("[SIM CONTR] Unknown command: {}. Type 'help' for available commands.", command);
                        }
                    }
                } else {
                    println!("[SIM CONTR] Failed to read command. Try again.");
                }
            }
    }
}