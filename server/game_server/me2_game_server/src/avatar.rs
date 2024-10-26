use crate::{
    connection::ConnectionID,
    proplist::{PropValue, Proplist},
    server::Server,
};

pub fn get_avatar_data(server: &Server, connection_id: ConnectionID) -> Option<String> {
    let connection = server.connections.get_connection(connection_id);

    let Some(player) = &connection.player else {
        return None;
    };

    let mut avatar_proplist = Proplist::new();
    avatar_proplist.add_element("dn", PropValue::String(player.session_id.clone()));
    avatar_proplist.add_element("cm", PropValue::String(player.customization.clone()));

    Some(format!(
        "A~{} {}\r",
        player.avatar_id,
        avatar_proplist.to_string()
    ))
}
