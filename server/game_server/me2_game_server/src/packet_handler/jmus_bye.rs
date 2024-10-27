use crate::{connection::ConnectionID, server::Server};

pub fn handle_jmus_bye(server: &mut Server, connection_id: ConnectionID) {
    server.connections.get_connection_mut(connection_id).kill();
}
