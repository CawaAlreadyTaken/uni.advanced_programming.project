pub mod gui {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    use crossbeam_channel::Receiver;
    use egui::*;
    use egui_glium::EguiGlium;
    use glium::glutin;
    use glium::Surface;
    use wg_2024::{config::Config, controller::NodeEvent};

    pub fn run_gui(
        topology: Arc<Config>,
        receiver_channel: Arc<Option<Receiver<NodeEvent>>>,
    ) {
        // Set up the graphical context
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("Node Visualization")
            .with_inner_size(glutin::dpi::LogicalSize::new(800.0, 600.0));
        let context_builder = glutin::ContextBuilder::new().with_vsync(true);
        let display =
            glium::Display::new(window_builder, context_builder, &event_loop).unwrap();

        let mut egui = EguiGlium::new(&display);

        // Initialize nodes and edges
        let mut node_positions = Vec::new();
        let mut edges = Vec::new();

        // Set initial positions for nodes
        let mut angle = 0.0;
        let angle_increment = 2.0 * std::f32::consts::PI
            / (topology.drone.len() + topology.client.len()) as f32;

        for _ in &topology.drone {
            let x = 0.5 + 0.4 * angle.cos();
            let y = 0.5 + 0.4 * angle.sin();
            node_positions.push((x, y));
            angle += angle_increment;
        }

        for _ in &topology.client {
            let x = 0.5 + 0.4 * angle.cos();
            let y = 0.5 + 0.4 * angle.sin();
            node_positions.push((x, y));
            angle += angle_increment;
        }

        // Start listening for messages in a separate thread
        if let Some(receiver) = Arc::clone(&receiver_channel).as_ref() {
            let receiver = Arc::clone(receiver);
            thread::spawn(move || {
                while let Ok(event) = receiver.recv() {
                    // Add logic to handle messages, e.g., create edges
                    match event {
                        NodeEvent::Link { from, to } => {
                            edges.push((from, to));
                        }
                        _ => {}
                    }
                }
            });
        }

        // Run the graphical server
        event_loop.run(move |event, _, control_flow| {
            *control_flow = glutin::event_loop::ControlFlow::Poll;

            match event {
                glutin::event::Event::RedrawEventsCleared => {
                    let mut target = display.draw();
                    target.clear_color(0.1, 0.1, 0.1, 1.0);

                    // Render with egui
                    egui.begin_frame(&display);
                    egui.ctx().area(
                        "Node Visualization",
                        egui::Area::new("node_visualization").fixed_pos(egui::Pos2::new(0.0, 0.0)),
                    );

                    // Draw nodes
                    for (i, &(x, y)) in node_positions.iter().enumerate() {
                        egui.ctx().circle(
                            Pos2::new(x * 800.0, y * 600.0),
                            10.0,
                            egui::Color32::WHITE,
                        );
                    }

                    // Draw edges
                    for &(from, to) in &edges {
                        if let (Some(&(fx, fy)), Some(&(tx, ty))) =
                            (node_positions.get(from), node_positions.get(to))
                        {
                            egui.ctx().line(
                                [Pos2::new(fx * 800.0, fy * 600.0), Pos2::new(tx * 800.0, ty * 600.0)],
                                (1.0, egui::Color32::GRAY),
                            );
                        }
                    }

                    egui.end_frame(&display);
                    egui.paint(&display, &mut target);
                    target.finish().unwrap();
                }
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit
                    }
                    _ => {}
                },
                _ => {}
            }

            // Artificial delay to reduce CPU usage
            thread::sleep(Duration::from_millis(16));
        });
    }
}
