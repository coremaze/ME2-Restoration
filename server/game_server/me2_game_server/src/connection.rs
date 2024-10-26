use std::collections::HashMap;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};

use crate::player::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionID(u64);
impl std::fmt::Display for ConnectionID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ConnectionID {
    pub fn to_num(&self) -> u64 {
        self.0
    }
}

pub struct Connection {
    stream: TcpStream,
    addr: SocketAddr,
    buffer: Vec<u8>,

    pub state: ConnectionState,
    pub current_packet_type: Option<String>,
    pub player: Option<Player>,
}

impl Connection {
    pub fn new(stream: TcpStream, addr: SocketAddr) -> Connection {
        stream
            .set_nonblocking(true)
            .expect("Cannot set non-blocking");
        Connection {
            stream,
            addr,
            buffer: Vec::new(),
            state: ConnectionState::New,
            player: None,
            current_packet_type: None,
        }
    }

    pub fn recv(&mut self) -> Result<Option<String>, std::io::Error> {
        let mut buffer = [0; 512];

        let read_result = self.stream.read(&mut buffer);

        if let Ok(n) = read_result {
            self.buffer.extend_from_slice(&buffer[..n]);
        }

        // Process complete packets terminated with \r
        let carriage_return_pos = self.buffer.iter().position(|&x| x == b'\r');

        if let Some(pos) = carriage_return_pos {
            let packet = self.buffer.drain(..=pos).collect::<Vec<u8>>();
            return Ok(Some(String::from_utf8_lossy(&packet[..pos]).to_string()));
        }

        if let Err(e) = read_result {
            if e.kind() != ErrorKind::WouldBlock {
                return Err(e);
            }
        }

        Ok(None)
    }

    pub fn send(&mut self, message: &str) -> Result<(), std::io::Error> {
        self.stream.write_all(message.as_bytes())?;
        Ok(())
    }
}

pub struct Connections {
    connections: HashMap<ConnectionID, Connection>,
    next_connection_id: u64,
}

impl Connections {
    pub fn new() -> Connections {
        Connections {
            connections: HashMap::new(),
            next_connection_id: 1,
        }
    }

    pub fn add_connection(&mut self, connection: Connection) -> ConnectionID {
        let connection_id = ConnectionID(self.next_connection_id);
        self.next_connection_id = self
            .next_connection_id
            .checked_add(1)
            .expect("Connection ID overflow");
        self.connections.insert(connection_id, connection);
        connection_id
    }

    pub fn remove_connection(&mut self, connection_id: ConnectionID) {
        self.connections.remove(&connection_id);
    }

    pub fn get_connection(&self, connection_id: ConnectionID) -> Option<&Connection> {
        self.connections.get(&connection_id)
    }

    pub fn get_connection_mut(&mut self, connection_id: ConnectionID) -> Option<&mut Connection> {
        self.connections.get_mut(&connection_id)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, ConnectionID, Connection> {
        self.connections.iter()
    }

    pub fn iter_mut(
        &mut self,
    ) -> std::collections::hash_map::IterMut<'_, ConnectionID, Connection> {
        self.connections.iter_mut()
    }
}

#[derive(Debug, PartialEq)]
pub enum ConnectionState {
    New,
    Checked,
    Authing,
    Authenticated,
}
