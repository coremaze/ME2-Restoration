use std::collections::HashMap;
use std::io::ErrorKind;
use std::io::Read;
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};

use super::connection::{Connection, ConnectionID, Connections};
use crate::connection::ConnectionState;
use crate::player::Player;
use crate::proplist::PropValue;
use crate::proplist::Proplist;

pub struct Server {
    listener: TcpListener,
    connections: Connections,
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
        let mut received_packets_and_ids: Vec<(ConnectionID, String)> = Vec::new();
        for (connection_id, connection) in self.connections.iter_mut() {
            match connection.recv() {
                Ok(Some(packet)) => received_packets_and_ids.push((*connection_id, packet)),
                Ok(None) => {}
                Err(e) => eprintln!("Error reading from stream: {}", e),
            }
        }

        if !received_packets_and_ids.is_empty() {
            println!("Received packets: {received_packets_and_ids:?}");
        }

        for (connection_id, packet) in received_packets_and_ids {
            handle_packet(self, connection_id, &packet);
        }
    }
}

fn handle_setup_packets(server: &mut Server, connection_id: ConnectionID, packet: &str) {
    let connection = server.connections.get_connection(connection_id).unwrap();

    match &connection.state {
        ConnectionState::New => {
            if packet == "JMUS_CHECK" {
                println!("JMUS_CHECK received");
                let connection = server
                    .connections
                    .get_connection_mut(connection_id)
                    .unwrap();
                connection.state = ConnectionState::Checked;
                connection.send("ALIVE").ok();
            }
        }
        ConnectionState::Checked => {
            if packet == "JMUS_AUTH" {
                println!("JMUS_AUTH received");
                let connection = server
                    .connections
                    .get_connection_mut(connection_id)
                    .unwrap();
                connection.state = ConnectionState::Authing;
            }
        }
        ConnectionState::Authing => {
            let session_id = packet.to_owned();

            let mut session_id_already_exists = false;
            for (_id, connection) in server.connections.iter_mut() {
                let Some(player) = &connection.player else {
                    continue;
                };

                if player.session_id == session_id {
                    session_id_already_exists = true;
                    break;
                }
            }

            let connection = server
                .connections
                .get_connection_mut(connection_id)
                .unwrap();

            let response = match session_id_already_exists {
                true => "INVALID",
                false => {
                    println!("Session ID: {session_id}");
                    connection.player = Some(Player::new(session_id));
                    connection.state = ConnectionState::Authenticated;
                    "VALID"
                }
            };

            connection.send(response).ok();
        }
        ConnectionState::Authenticated => {}
    }
}

fn handle_packet(server: &mut Server, connection_id: ConnectionID, packet: &str) {
    if packet.is_empty() {
        return;
    }

    println!("Handling packet: {packet}");

    let previous_type = server
        .connections
        .get_connection(connection_id)
        .unwrap()
        .current_packet_type
        .clone();

    handle_setup_packets(server, connection_id, &packet);

    let mut resp: Option<String> = None;
    // Only authenticated packets past here
    if let ConnectionState::Authenticated = &server
        .connections
        .get_connection(connection_id)
        .unwrap()
        .state
    {
        println!("Authenticated packet: {packet} - {previous_type:?}");
        if packet == "US" {
            let Some(player) = &server
                .connections
                .get_connection(connection_id)
                .unwrap()
                .player
            else {
                eprintln!("US packet: {packet}");
                return;
            };
            let dn = player.session_id.clone();
            let cm = "241111112111111";
            let uid = connection_id.to_string();
            resp = Some(format!(
                "U [#uid: {uid}, #dn: \"{dn}\", #iV: [], #lg: \"en\", #mm: 0, #cm: \"{cm}\", #uc: [], #uh: [], #bl: [], #bu: [], #cb: \"\", #pa: [], #pp: 0, #ppnew: 0, #gt: 0, #ga: 0, #hs: []]"
            ));
        } else if let Some(previous_type) = previous_type {
            match previous_type.as_str() {
                "MU" => {
                    resp = handle_mu_packet(server, connection_id, packet);
                    inform_all_avatars(server, connection_id);
                }
                _ => {}
            }
            server
                .connections
                .get_connection_mut(connection_id)
                .unwrap()
                .current_packet_type = None;
        }
    } else {
        return;
    }

    if let Some(resp) = resp {
        server
            .connections
            .get_connection_mut(connection_id)
            .unwrap()
            .send(&resp)
            .ok();
    } else if packet.len() == 2 {
        // Some packets come over as multiple lines
        server
            .connections
            .get_connection_mut(connection_id)
            .unwrap()
            .current_packet_type = Some(packet.to_owned());
    }
}

fn handle_mu_packet(
    server: &mut Server,
    connection_id: ConnectionID,
    packet: &str,
) -> Option<String> {
    let proplist_fixed = format!("[{packet}]");
    let Ok(proplist) = Proplist::parse(&proplist_fixed) else {
        eprintln!("Failed to parse proplist: {proplist_fixed}");
        return None;
    };
    let Some(PropValue::Vector(location)) = proplist.get_element("l") else {
        eprintln!("3 MU packet: {proplist:#?}");
        return None;
    };

    let Some(PropValue::Vector(rotation)) = proplist.get_element("r") else {
        eprintln!("2 MU packet: {proplist:#?}");
        return None;
    };

    println!("Location: {location:?}");
    println!("Rotation: {rotation:?}");

    let connection = server
        .connections
        .get_connection_mut(connection_id)
        .unwrap();
    let Some(player) = &mut connection.player else {
        eprintln!("1 MU packet: {proplist:#?}");
        return None;
    };

    player.location = *location;
    player.rotation = *rotation;

    // println!("MU packet: {proplist:#?}");

    None
}

fn inform_all_avatars(server: &mut Server, connection_id: ConnectionID) {
    let mut movement_list = Vec::new();
    let mut avatar_packets = Vec::new();
    for (connection_id, connection) in server.connections.iter_mut() {
        let Some(player) = &connection.player else {
            continue;
        };

        let mut movement_proplist = Proplist::new();
        movement_proplist.add_element("i", PropValue::Integer(connection_id.to_num() as i64));
        movement_proplist.add_element("l", PropValue::Vector(player.location));
        movement_proplist.add_element("r", PropValue::Vector(player.rotation));
        movement_proplist.add_element("ic", PropValue::Integer(0));
        movement_proplist.add_element("gs", PropValue::Integer(1));
        movement_proplist.add_element("gs", PropValue::Integer(1));
        movement_proplist.add_element("ar", PropValue::Float(0.15));
        movement_list.push(PropValue::Proplist(movement_proplist));

        let mut avatar_proplist = Proplist::new();
        avatar_proplist.add_element("dn", PropValue::String(player.session_id.clone()));
        avatar_proplist.add_element("cm", PropValue::String("241111112111111".to_string()));

        let id = connection_id.to_string();
        let avatar_packet = format!("A~{id} {}\r", avatar_proplist.to_string());
        avatar_packets.push(avatar_packet);
    }
    let movement_propvaluelist = PropValue::List(movement_list);

    println!("Inform all avatars: {}", movement_propvaluelist.to_string());
    println!("Inform all avatars2: {avatar_packets:?}");

    let connection = server
        .connections
        .get_connection_mut(connection_id)
        .unwrap();
    for avatar_packet in &avatar_packets {
        connection.send(avatar_packet).ok();
    }
    connection.send(&movement_propvaluelist.to_string()).ok();
}
