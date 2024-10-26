use crate::{connection::ConnectionID, server::Server};

pub fn handle_im_alive(server: &mut Server, connection_id: ConnectionID) {
    println!("IM_ALIVE packet received from {connection_id}");
}
