use crate::{connection::ConnectionID, packet::Ct, server::Server};

pub fn handle_ct(server: &mut Server, connection_id: ConnectionID, data: &Ct) {
    println!("Chat message: {:#?}", data);
}
