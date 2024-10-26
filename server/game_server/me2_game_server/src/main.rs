mod connection;
mod player;
mod proplist;
mod server;

use server::Server;

use std::collections::HashMap;
use std::io::ErrorKind;
use std::io::Read;
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};

fn main() {
    let mut server = Server::new();
    server.run();
}
