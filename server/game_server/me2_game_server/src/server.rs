use std::net::TcpListener;

use super::connection::{Connection, ConnectionID, Connections};
use crate::packet;
use crate::packet::CSPacket;
use crate::packet_handler::*;

pub struct Server {
    listener: TcpListener,
    pub connections: Connections,
}

impl Server {
    pub fn new() -> Server {
        Server {
            listener: TcpListener::bind("0.0.0.0:7158").expect("Failed to bind to port 7158"),
            connections: Connections::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.accept_connections();
            self.process_connections();
        }
    }

    fn accept_connections(&mut self) {
        match self.listener.set_nonblocking(true) {
            Ok(_) => match self.listener.accept() {
                Ok((stream, addr)) => {
                    println!("New connection from {}", addr);
                    let connection_id = self
                        .connections
                        .add_connection(Connection::new(stream, addr));
                    println!("Connection ID: {connection_id}");
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No connection is ready to be accepted
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            },
            Err(e) => {
                eprintln!("Error setting non-blocking: {}", e);
            }
        }
    }

    fn process_connections(&mut self) {
        for (connection_id, connection) in self.connections.iter_mut() {
            connection.recv().ok();
        }

        let mut received_packets_and_ids: Vec<(ConnectionID, CSPacket)> = Vec::new();
        for (&connection_id, connection) in self.connections.iter_mut() {
            let packet = packet::take_packet(&mut connection.buffer);

            if let Some(packet) = packet {
                println!("Packet: {packet:?}");
                received_packets_and_ids.push((connection_id, packet));
            }
        }

        for (connection_id, packet) in received_packets_and_ids {
            handle_packet(self, connection_id, &packet);
        }
    }
}
fn handle_packet(server: &mut Server, connection_id: ConnectionID, packet: &CSPacket) {
    match packet {
        CSPacket::JmusCheck => handle_jmus_check(server, connection_id, packet),
        CSPacket::JmusAuth(data) => handle_jmus_auth(server, connection_id, data),
        CSPacket::Us => handle_us(server, connection_id, packet),
        CSPacket::Uu(settings) => handle_uu(server, connection_id, settings),
        CSPacket::Mu(data) => handle_mu(server, connection_id, data),
        CSPacket::Ct(data) => handle_ct(server, connection_id, data),
        CSPacket::ImAlive => handle_im_alive(server, connection_id),
        CSPacket::Gp(avatar_id) => handle_gp(server, connection_id, avatar_id),
    }
}
