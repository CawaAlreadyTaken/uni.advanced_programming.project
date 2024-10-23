/* Example

pub mod api;
pub mod connection;

pub struct Client {
    base_url: String,
    connection: connection::Connection,
}

impl Client {
    pub fn new(base_url: &str) -> Client {
        Client {
            base_url: base_url.to_string(),
            connection: connection::Connection::new(),
        }
    }

    pub fn get(&self, endpoint: &str) -> Result<String, String> {
        let url = format!("{}/{}", self.base_url, endpoint);
        self.connection.get(&url)
    }

    pub fn post(&self, endpoint: &str, data: &str) -> Result<String, String> {
        let url = format!("{}/{}", self.base_url, endpoint);
        self.connection.post(&url, data)
    }
}
 */