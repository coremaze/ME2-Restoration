use crate::{
    avatar::get_avatar_data,
    connection::ConnectionID,
    packet::Mu,
    proplist::{PropValue, Proplist},
    server::Server,
};

pub fn handle_mu(server: &mut Server, connection_id: ConnectionID, data: &Mu) {
    let fixed_movement_prop_list = format!("[{}]", data.movement);
    let Ok(movement_prop_list) = Proplist::parse(&fixed_movement_prop_list) else {
        println!("Invalid movement prop list: {fixed_movement_prop_list}");
        return;
    };

    let connection = server.connections.get_connection_mut(connection_id);

    let Some(player) = &mut connection.player else {
        println!("Player not found");
        return;
    };

    if let Some(PropValue::Vector(location)) = movement_prop_list.get_element("l") {
        player.location = *location;
    }

    if let Some(PropValue::Vector(rotation)) = movement_prop_list.get_element("r") {
        player.rotation = *rotation;
    }

    // println!("MU Movement packet received: {movement_prop_list:?}");

    // Inform this player about all other players' movement
    inform_all_avatars(server, connection_id);
}

fn inform_all_avatars(server: &mut Server, connection_id: ConnectionID) {
    let mut movement_list = Vec::new();
    for (connection_id, connection) in server.connections.iter_mut() {
        let Some(player) = &connection.player else {
            continue;
        };

        let mut movement_proplist = Proplist::new();
        movement_proplist.add_element("i", PropValue::Integer(player.avatar_id as i64));
        movement_proplist.add_element("l", PropValue::Vector(player.location));
        movement_proplist.add_element("r", PropValue::Vector(player.rotation));
        movement_proplist.add_element("ic", PropValue::Integer(0));
        movement_proplist.add_element("gs", PropValue::Integer(1));
        movement_proplist.add_element("gs", PropValue::Integer(1));
        movement_proplist.add_element("ar", PropValue::Float(0.15));
        movement_list.push(PropValue::Proplist(movement_proplist));
    }
    let movement_propvaluelist = PropValue::List(movement_list);

    println!("Inform all avatars: {}", movement_propvaluelist.to_string());

    let connection = server.connections.get_connection_mut(connection_id);

    // Send all other player's avatar data to the new player
    let mut avatar_datas: Vec<String> = Vec::new();
    for (&other_connection_id, other_connection) in server.connections.iter() {
        if other_connection_id == connection_id {
            continue;
        }
        if let Some(avatar_data) = get_avatar_data(server, other_connection_id) {
            avatar_datas.push(avatar_data);
        }
    }

    let connection = server.connections.get_connection_mut(connection_id);
    for avatar_data in avatar_datas {
        connection.send(&avatar_data).ok();
    }
    connection.send(&movement_propvaluelist.to_string()).ok();
}