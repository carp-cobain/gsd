use crate::config::Config;
use std::{io, net::SocketAddr};
use tokio::net::{TcpListener, TcpSocket};

impl Config {
    pub fn tcp_listener(&self) -> TcpListener {
        let addr: SocketAddr = self
            .listen_addr
            .parse()
            .expect("Failed to parse listen address");

        reuse_listener(addr).expect("Failed calling reuse_listener")
    }
}

// See:
// https://github.com/TechEmpower/FrameworkBenchmarks/blob/master/frameworks/Rust/axum/src/server.rs#L21
// Not sure this is necessary
// NOTE: this uses unsafe calls
fn reuse_listener(addr: SocketAddr) -> io::Result<TcpListener> {
    let socket = match addr {
        SocketAddr::V4(_) => TcpSocket::new_v4()?,
        SocketAddr::V6(_) => TcpSocket::new_v6()?,
    };
    #[cfg(unix)]
    {
        log::debug!("cfg(unix): calling set_reuseport on socket");
        if let Err(e) = socket.set_reuseport(true) {
            log::warn!("error setting SO_REUSEPORT: {}", e);
        }
    }
    if let Err(e) = socket.set_reuseaddr(true) {
        log::warn!("error calling set_reuseaddr: {}", e);
    }
    if let Err(e) = socket.set_nodelay(true) {
        log::warn!("error calling set_nodelay: {}", e);
    }
    socket.bind(addr)?;
    socket.listen(1024)
}
