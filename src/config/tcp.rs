use crate::config::Config;
use std::{io, net::SocketAddr};
use tokio::net::{TcpListener, TcpSocket};

impl Config {
    pub fn tcp_listener(&self) -> TcpListener {
        let addr: SocketAddr = self
            .listen_addr
            .parse()
            .expect("Failed to parse listen address");

        reuse_listener(addr).expect("Failed to re-use socket address")
    }
}

// See:
// https://github.com/TechEmpower/FrameworkBenchmarks/blob/master/frameworks/Rust/axum/src/server.rs#L21

fn reuse_listener(addr: SocketAddr) -> io::Result<TcpListener> {
    let socket = match addr {
        SocketAddr::V4(_) => TcpSocket::new_v4()?,
        SocketAddr::V6(_) => TcpSocket::new_v6()?,
    };
    #[cfg(unix)]
    {
        if let Err(e) = socket.set_reuseport(true) {
            log::error!("error setting SO_REUSEPORT: {}", e);
        }
    }
    socket.set_nodelay(true).expect("Failed setting nodelay");
    socket.set_reuseaddr(true)?;
    socket.bind(addr)?;
    socket.listen(1024)
}
