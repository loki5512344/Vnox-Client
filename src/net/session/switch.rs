/// User switched node while connected (`NetCommand::Connect`).
#[derive(Debug)]
pub struct ReconnectRequested {
    pub address: String,
    pub auto_reconnect: bool,
}

impl std::fmt::Display for ReconnectRequested {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "reconnect to {}", self.address)
    }
}

impl std::error::Error for ReconnectRequested {}

pub struct SessionTarget {
    pub address: String,
    pub auto_reconnect: bool,
    pub channels: Vec<String>,
}
