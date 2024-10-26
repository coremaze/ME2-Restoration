mod avatar;
mod connection;
mod packet;
mod packet_handler;
mod player;
mod proplist;
mod server;

use server::Server;

fn main() {
    let mut server = Server::new();
    server.run();
}
