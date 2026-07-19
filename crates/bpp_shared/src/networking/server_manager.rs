/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/

use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};

use crate::logger::logger::global_logger;
use crate::networking::network_stream::NetworkStream;

// Should act as a singleton
pub struct ServerManager {
    server_socket: Option<TcpListener>,

    pub streams: Vec<NetworkStream>,
}

impl ServerManager {
    pub fn new(port: u16) -> Self {
        let mut server_manager = ServerManager {
            server_socket: None,
            streams: Vec::new(),
        };

        // Only bind new socket if it doesn't exist already
        if server_manager.server_socket.is_some() {
            return server_manager;
        }

        let address = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
        match TcpListener::bind(address) {
            Ok(listener) => {
                server_manager.server_socket = Some(listener);
            }
            Err(e) => {
                global_logger().error(format!("Bind error: {e}\n"));
                return server_manager;
            }
        }

        server_manager
    }

    // Init network stream
    pub fn init_connection(&mut self) -> bool {
        if let Some(listener) = self.server_socket.as_ref() {
            if let Ok((client_socket, _)) = listener.accept() {
                self.streams.push(NetworkStream::new(client_socket));
                return true;
            }
        }
        false
    }
}

impl Drop for ServerManager {
    fn drop(&mut self) {
        // Clear all streams, which should close them
        self.streams.clear();
        // Close server
        self.server_socket = None;
    }
}
