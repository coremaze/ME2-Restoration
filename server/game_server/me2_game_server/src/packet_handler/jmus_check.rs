use crate::{connection::ConnectionID, packet::CSPacket, server::Server};

pub fn handle_jmus_check(server: &mut Server, connection_id: ConnectionID, _packet: &CSPacket) {
    let connection = server.connections.get_connection_mut(connection_id);
    connection.send("ALIVE").ok();
}
