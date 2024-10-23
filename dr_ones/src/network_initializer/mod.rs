struct NetworkInitializer {
    // ...
}

impl NetworkInitializer {
    pub fn new() -> Self {
        NetworkInitializer {}
    }

    pub fn start(&mut self) {
        loop {
            println!("NetworkInitializer started");
        }
    }
}

pub fn start_network_initializer() {
    let mut network_initializer = NetworkInitializer::new();
    network_initializer.start();
}
