use std::net::TcpListener;
use std::time::Instant;

use super::connection::{Connection, ConnectionID, Connections};
use crate::packet::client_packet;
use crate::packet::client_packet::CSPacket;
use crate::packet::server_packet::send_keepalive;
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
            self.send_keepalives();
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
            if let Err(e) = connection.recv() {
                eprintln!("Error receiving from connection {connection_id}: {e}");
                connection.kill();
            }
        }

        let mut received_packets_and_ids: Vec<(ConnectionID, CSPacket)> = Vec::new();
        for (&connection_id, connection) in self.connections.iter_mut() {
            let packet = client_packet::take_packet(&mut connection.buffer);

            if let Some(packet) = packet {
                // println!("Packet: {packet:?}");
                received_packets_and_ids.push((connection_id, packet));
            }
        }

        for (connection_id, packet) in received_packets_and_ids {
            handle_packet(self, connection_id, &packet);
        }

        for (cid, connection) in self.connections.iter_mut() {
            if connection.is_killed() {
                println!("Killing connection {}", cid);
            }
        }
        self.connections.remove_dead();
    }

    fn send_keepalives(&mut self) {
        let now = Instant::now();
        for (_, connection) in self.connections.iter_mut() {
            if now.duration_since(connection.last_sent_keepalive).as_secs() > 10 {
                send_keepalive(connection);
            }
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
        CSPacket::JmusBye => handle_jmus_bye(server, connection_id),
    }
}
