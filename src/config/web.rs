use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Web {
    pub enabled: bool,
    pub address: Option<SocketAddr>,
}
