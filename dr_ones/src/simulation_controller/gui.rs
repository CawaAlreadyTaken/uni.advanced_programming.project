pub mod gui {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::thread;

    use macroquad::prelude::*;
    use wg_2024::{config::Config, controller::DroneEvent};
    use crossbeam_channel::Receiver;

    pub async fn run_gui(
        topology: Arc<Config>,
        receiver_channel: Arc<Option<Receiver<DroneEvent>>>,
    ) {
        // Shared state for drawing lines
        let lines = Arc::new(Mutex::new(Vec::<i32>::new()));

        // Spawn a thread to listen for incoming NodeEvent messages
        let receiver_channel_clone = Arc::clone(&receiver_channel);
        let lines_clone = Arc::clone(&lines);
        thread::spawn(move || {
            if let Some(receiver) = receiver_channel_clone.as_ref() {
                while let Ok(event) = receiver.recv() {
                    // Process the event to add lines or update state
                    match event {
                        DroneEvent::PacketSent ( packet ) => {
                            // Add a line to the shared state
                            //let mut lines = lines_clone.lock().unwrap();
                            //lines.push((from, to));
                        }
                        _ => {}
                    }
                }
            }
        });

        // Pre-compute positions for nodes
        let mut positions = Vec::new();
        let mut next_position = 50.0;

        for drone in &topology.drone {
            positions.push((next_position, 100.0)); // Example positions
            next_position += 100.0;
        }
        for client in &topology.client {
            positions.push((next_position, 200.0));
            next_position += 100.0;
        }
        for server in &topology.server {
            positions.push((next_position, 300.0));
            next_position += 100.0;
        }

        // Rendering loop
        loop {
            clear_background(BLACK);

            // Draw nodes
            for (i, position) in positions.iter().enumerate() {
                let drone_end = topology.drone.len();
                let client_end = drone_end + topology.client.len();

                let color;
                if i < drone_end {
                    color = BLUE;
                } else if i <= client_end {
                    color = GREEN;
                } else {
                    color = RED;
                }

                draw_circle(position.0, position.1, 20.0, color);
            }

            // Draw lines from the shared state
            let positions: HashMap<usize, (f32, f32)> = HashMap::new();
            let lines: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());

            let lines = lines.lock().unwrap();
            for (from, to) in lines.iter() {
                if let (Some(from_pos), Some(to_pos)) = (
                    positions.get(from), // `from` e `to` sono `usize`
                    positions.get(to),
                ) {
                    draw_line(from_pos.0, from_pos.1, to_pos.0, to_pos.1, 2.0, WHITE);
                }
            }

            // Next frame
            next_frame().await;
        }
    }
}
