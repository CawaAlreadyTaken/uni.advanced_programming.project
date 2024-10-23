//use crate::client;

struct SimulationController {
    // ...
}

impl SimulationController {
    pub fn new() -> Self {
        SimulationController{}
    }

    pub fn start(&mut self) {
        loop {
            println!("SimulationController started");
        }
    }
}

pub fn start_simulation_controller() {
    let mut simulation_controller = SimulationController::new();
    simulation_controller.start();
}
