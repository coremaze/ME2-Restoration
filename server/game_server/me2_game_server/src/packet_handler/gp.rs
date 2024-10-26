use crate::{connection::ConnectionID, server::Server};

pub fn handle_gp(server: &mut Server, connection_id: ConnectionID, avatar_id: &str) {
    println!("Handling GP packet for avatar_id: {avatar_id}");
}
