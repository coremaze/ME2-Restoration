use crate::packet::server_packet::send_alive;
use crate::{connection::ConnectionID, packet::client_packet::CSPacket, server::Server};

pub fn handle_jmus_check(server: &mut Server, connection_id: ConnectionID, _packet: &CSPacket) {
    send_alive(server.connections.get_connection_mut(connection_id));
}
