use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub(crate) enum Connection {
    #[serde(rename_all = "kebab-case")]
    Tcp { addr: SocketAddr },
    #[serde(rename_all = "kebab-case")]
    Udp {
        bind: Option<SocketAddr>,
        addr: SocketAddr,
    },
    #[serde(rename_all = "kebab-case")]
    Relay {
        auto_select_host: Option<String>,
        websocket: bool,
        admin: Option<String>,
        spectator: Option<String>,
    },
}

impl Connection {
    pub(crate) fn as_insim_builder(&self) -> insim::builder::Builder {
        match self {
            Connection::Relay {
                auto_select_host,
                websocket,
                spectator,
                admin,
            } => insim::relay()
                .relay_select_host(auto_select_host.clone())
                .relay_websocket(*websocket)
                .relay_spectator_password(spectator.clone())
                .relay_admin_password(admin.clone()),
            Connection::Tcp { addr } => insim::tcp(*addr),

            Connection::Udp { bind, addr } => insim::udp(*addr, *bind),
        }
    }
}
