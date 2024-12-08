//! Graphical User Interface module for the simulation controller.
//! Provides real-time visualization of the network topology and packet flow.

pub mod gui {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use crossbeam_channel::Receiver;
    use macroquad::prelude::*;
    use wg_2024::{config::Config, controller::DroneEvent};

    /// Core rendering colors
    const BACKGROUND_COLOR: Color = BLACK;
    const DRONE_COLOR: Color = BLUE;
    const CLIENT_COLOR: Color = GREEN;
    const SERVER_COLOR: Color = RED;
    const CONNECTION_COLOR: Color = WHITE;
    const NODE_RADIUS: f32 = 20.0;
    const LINE_THICKNESS: f32 = 2.0;

    /// Represents a position in the visualization
    #[derive(Clone, Copy)]
    struct Position {
        x: f32,
        y: f32,
    }

    /// Runs the GUI visualization
    pub async fn run_gui(
        topology: Arc<Config>,
        receiver_channel: Arc<Option<Receiver<DroneEvent>>>,
    ) {
        // Shared state for drawing lines
        let active_connections = Arc::new(Mutex::new(Vec::<(usize, usize)>::new()));
        
        // Start the event listener thread
        spawn_event_listener(Arc::clone(&receiver_channel), Arc::clone(&active_connections));

        // Pre-compute node positions
        let node_positions = compute_node_positions(&topology);

        // Main rendering loop
        run_render_loop(topology, node_positions, active_connections).await;
    }

    /// Spawns a thread to listen for network events
    fn spawn_event_listener(
        receiver_channel: Arc<Option<Receiver<DroneEvent>>>,
        active_connections: Arc<Mutex<Vec<(usize, usize)>>>,
    ) {
        thread::spawn(move || {
            if let Some(receiver) = receiver_channel.as_ref() {
                while let Ok(event) = receiver.recv() {
                    handle_network_event(event, &active_connections);
                }
            }
        });
    }

    /// Handles incoming network events
    fn handle_network_event(
        event: DroneEvent,
        active_connections: &Arc<Mutex<Vec<(usize, usize)>>>,
    ) {
        match event {
            DroneEvent::PacketSent(packet) => {
                // TODO: Update active connections based on packet routing
            },
            _ => {},
        }
    }

    /// Computes positions for all nodes in the visualization
    fn compute_node_positions(topology: &Config) -> Vec<Position> {
        let mut positions = Vec::new();
        let mut next_x = 50.0;

        // Position drones on top row
        for _ in &topology.drone {
            positions.push(Position { x: next_x, y: 100.0 });
            next_x += 100.0;
        }

        // Position clients in middle row
        for _ in &topology.client {
            positions.push(Position { x: next_x, y: 200.0 });
            next_x += 100.0;
        }

        // Position servers on bottom row
        for _ in &topology.server {
            positions.push(Position { x: next_x, y: 300.0 });
            next_x += 100.0;
        }

        positions
    }

    /// Main rendering loop
    async fn run_render_loop(
        topology: Arc<Config>,
        positions: Vec<Position>,
        active_connections: Arc<Mutex<Vec<(usize, usize)>>>,
    ) {
        loop {
            clear_background(BACKGROUND_COLOR);

            // Draw nodes
            draw_nodes(&topology, &positions);

            // Draw active connections
            draw_connections(&positions, &active_connections);

            next_frame().await;
        }
    }

    /// Draws all nodes in the network
    fn draw_nodes(topology: &Config, positions: &[Position]) {
        let mut index = 0;

        // Draw drones
        for _ in &topology.drone {
            draw_node(positions[index], DRONE_COLOR);
            index += 1;
        }

        // Draw clients
        for _ in &topology.client {
            draw_node(positions[index], CLIENT_COLOR);
            index += 1;
        }

        // Draw servers
        for _ in &topology.server {
            draw_node(positions[index], SERVER_COLOR);
            index += 1;
        }
    }

    /// Draws a single node
    fn draw_node(pos: Position, color: Color) {
        draw_circle(pos.x, pos.y, NODE_RADIUS, color);
    }

    /// Draws active connections between nodes
    fn draw_connections(
        positions: &[Position],
        active_connections: &Arc<Mutex<Vec<(usize, usize)>>>,
    ) {
        if let Ok(connections) = active_connections.lock() {
            for (from, to) in connections.iter() {
                let from_pos = positions[*from];
                let to_pos = positions[*to];
                
                draw_line(
                    from_pos.x,
                    from_pos.y,
                    to_pos.x,
                    to_pos.y,
                    LINE_THICKNESS,
                    CONNECTION_COLOR,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO: Add tests for position computation and event handling
}