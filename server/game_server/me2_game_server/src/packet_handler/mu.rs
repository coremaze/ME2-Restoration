use crate::packet::server_packet::send_avatar;
use crate::{
    connection::ConnectionID,
    packet::client_packet::Mu,
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

    if let Some(location) = movement_prop_list.get_vector("l") {
        player.location = location;
    }

    if let Some(rotation) = movement_prop_list.get_vector("r") {
        player.rotation = rotation;
    }

    if let Some(animation_state) = movement_prop_list.get_string("as") {
        player.animation_state = animation_state.to_string();
    }

    if let Some(animation_rate) = movement_prop_list.get_number("ar") {
        player.animation_rate = animation_rate as f32;
    }

    // println!("MU Movement packet received: {movement_prop_list:?}");

    // Inform this player about all other players' movement
    inform_all_avatars(server, connection_id);
}

fn inform_all_avatars(server: &mut Server, connection_id: ConnectionID) {
    let mut movement_list = Vec::new();
    for (_, connection) in server.connections.iter_mut() {
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
        movement_proplist.add_element("ar", PropValue::Float(player.animation_rate.into()));
        movement_proplist.add_element("as", PropValue::String(player.animation_state.clone()));
        movement_list.push(PropValue::Proplist(movement_proplist));
    }
    let movement_propvaluelist = PropValue::List(movement_list);

    // println!("Inform all avatars: {}", movement_propvaluelist.to_string());

    // Send all other player's avatar data to the new player
    let mut av_id_name_custs: Vec<(u32, String, String)> = Vec::new();
    for (&other_connection_id, other_connection) in server.connections.iter() {
        if other_connection_id == connection_id {
            continue;
        }
        if let Some(player) = &other_connection.player {
            av_id_name_custs.push((
                player.avatar_id,
                player.display_name.clone(),
                player.customization.clone(),
            ));
        }
    }

    let connection = server.connections.get_connection_mut(connection_id);
    for (avatar_id, display_name, customization) in av_id_name_custs {
        send_avatar(connection, avatar_id.into(), &display_name, &customization);
    }
    let movement_packet = format!("{}\r", movement_propvaluelist.to_string());
    connection.send(&movement_packet);
}
