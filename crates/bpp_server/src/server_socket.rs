/*
 * Copyright (c) 2026, Pixel Brush <pixelbrush.dev>
 * Copyright (c) 2026, Aidan <JcbbcEnjoyer>
 *
 * SPDX-License-Identifier: AGPL-3.0-only
 *
*/
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use bpp_shared::logger::logger::global_logger;

pub fn close_socket(socket: TcpListener) {
    drop(socket);
}

pub fn create_server_socket(port: u16) -> Option<TcpListener> {
    let server_socket = match TcpListener::bind(("0.0.0.0", port)) {
        Ok(server_socket) => server_socket,
        Err(_) => {
            global_logger().error("**** FAILED TO BIND SOCKET! ****\n");
            return None;
        }
    };

    if server_socket.set_nonblocking(true).is_err() {
        return None;
    }

    Some(server_socket)
}

pub fn create_client_socket(socket: &TcpListener) -> Option<TcpStream> {
    let (client_socket, _) = match socket.accept() {
        Ok(accepted) => accepted,
        Err(_) => return None,
    };

    let _ = client_socket.set_nonblocking(true);
    let _ = client_socket.set_read_timeout(Some(Duration::from_micros(45000)));

    Some(client_socket)
}
