use std::collections::HashMap;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Instant;

use crate::player::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionID(u64);
impl std::fmt::Display for ConnectionID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ConnectionID {
    pub fn to_num(self) -> u64 {
        self.0
    }
}

pub struct Connection {
    stream: TcpStream,
    addr: SocketAddr,
    killed: bool,
    pub buffer: Vec<u8>,
    pub last_sent_keepalive: Instant,

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
            killed: false,
            buffer: Vec::new(),
            last_sent_keepalive: Instant::now(),
            player: None,
        }
    }

    pub fn recv(&mut self) -> std::io::Result<usize> {
        let mut buffer = [0; 512];

        match self.stream.read(&mut buffer) {
            Ok(n) => {
                self.buffer.extend_from_slice(&buffer[..n]);
                Ok(n)
            }
            Err(e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    return Err(e);
                }
                Ok(0)
            }
        }
    }

    pub fn send(&mut self, message: &str) {
        // println!("Sending message: {}", message);
        if let Err(e) = self.stream.write_all(message.as_bytes()) {
            println!("Error sending message: {:?}", e);
            self.kill();
        }
    }

    pub fn kill(&mut self) {
        self.stream.shutdown(std::net::Shutdown::Both).ok();
        self.killed = true;
    }

    pub fn is_killed(&self) -> bool {
        self.killed
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
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

    pub fn get_connection(&self, connection_id: ConnectionID) -> &Connection {
        self.connections.get(&connection_id).unwrap_or_else(|| {
            panic!(
                "get_connection was passed a non-existent ID: {}",
                connection_id
            )
        })
    }

    pub fn get_connection_mut(&mut self, connection_id: ConnectionID) -> &mut Connection {
        self.connections.get_mut(&connection_id).unwrap_or_else(|| {
            panic!(
                "get_connection_mut was passed a non-existent ID: {}",
                connection_id
            )
        })
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, ConnectionID, Connection> {
        self.connections.iter()
    }

    pub fn iter_mut(
        &mut self,
    ) -> std::collections::hash_map::IterMut<'_, ConnectionID, Connection> {
        self.connections.iter_mut()
    }

    pub fn remove_dead(&mut self) {
        self.connections
            .retain(|_, connection| !connection.is_killed());
    }
}
