use std::time::Instant;

use crate::{
    connection::Connection,
    proplist::{PropValue, Proplist},
};

pub fn send_chat(connection: &mut Connection, username: &str, message: &str) {
    let packet = format!("L~[\"{username}: {message}\"]\r");
    connection.send(&packet);
}

pub fn send_server_message(connection: &mut Connection, message: &str) {
    let packet = format!("L~[\"{message}\"]\r");
    connection.send(&packet);
}

pub fn send_chat_alert(connection: &mut Connection, message: &str) {
    let packet = format!("K~\n\n{message}\r");
    connection.send(&packet);
}

pub enum AdminCommand {
    Mute,
    Unmute,
    Kill,
}

pub fn send_admin_command(connection: &mut Connection, command: AdminCommand) {
    let command_str = match command {
        AdminCommand::Mute => "MUTE",
        AdminCommand::Unmute => "UNMUTE",
        AdminCommand::Kill => "KILL",
    };
    let packet = format!("X~{command_str}\r");
    connection.send(&packet);
}

pub fn send_avatar(
    connection: &mut Connection,
    avatar_id: i64,
    display_name: &str,
    customization: &str,
) {
    let mut avatar_proplist = Proplist::new();
    avatar_proplist.add_element("dn", PropValue::String(display_name.to_string()));
    avatar_proplist.add_element("cm", PropValue::String(customization.to_string()));

    let packet = format!("A~{} {}\r", avatar_id, avatar_proplist);

    connection.send(&packet);
}

pub fn send_auth_response(connection: &mut Connection, valid: bool) {
    let resp = if valid { "VALID" } else { "BYE" };
    connection.send(resp);
}

pub fn send_alive(connection: &mut Connection) {
    connection.send("ALIVE");
}

pub fn send_keepalive(connection: &mut Connection) {
    // Not a standard packet but it helps kill connections that have ended
    println!("Sending keepalive");
    connection.last_sent_keepalive = Instant::now();
    connection.send("KEEPALIVE\r");
}
