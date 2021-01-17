use std::process::{Command, Stdio, Child};
use std::io::Error;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION lancement du processus matrix_commander

pub fn matrix_commander_daemon_launch() -> Result<Child, Error> {
    // _initialise le daemon matrix-commander
    let daemon = Command::new("./../matrix-commander/matrix-commander.py")
        .arg("-c./../matrix-commander/credentials.json")
        .arg("-s./../matrix-commander/store/")
        .arg("-lforever")
        .stdout(Stdio::piped())
        .spawn();
    daemon
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTIONS pour nettoyer les trames de matrix-commander

pub fn clean_room_origin(raw_room_origin:String) -> Result<String, String> {
    let debut = match raw_room_origin.find("room") {
        Some(debut_index) => debut_index + 5,
        None => return Err("ERROR: clean_room_origin start".to_string()),
    };
    let fin = match raw_room_origin.find("[") {
        Some(fin_index) => fin_index - 1,
        None => return Err("ERROR: clean_room_origin end".to_string()),
    };
    if debut >= fin {
        Err("ERROR: clean_room_origin matrix-commander output unreadable".to_string())
    }else {
        let clean_room_origin = &raw_room_origin[debut..fin];
        Ok(clean_room_origin.to_string())
    }
}

pub fn clean_room_id(raw_room_id:String) -> Result<String, String> {
    let debut = match raw_room_id.find("[") {
        Some(debut_index) => debut_index + 1,
        None => return Err("ERROR: clean_room_id start".to_string()),
    };
    let fin = match raw_room_id.find("]") {
        Some(fin_index) => fin_index,
        None => return Err("ERROR: clean_room_id end".to_string()),
    };
    if debut >= fin {
        Err("ERROR: clean_room_id matrix-commander output unreadable".to_string())
    } else {
        let clean_room_id = &raw_room_id[debut..fin];
        Ok(clean_room_id.to_string())
    }
}

pub fn clean_sender_id(raw_sender_id:String) -> Result<String, String> {
    let debut = match raw_sender_id.find("[") {
        Some(debut_index) => debut_index + 1,
        None => return Err("ERROR: clean_sender_id start".to_string()),
    };
    let fin = match raw_sender_id.find("]") {
        Some(fin_index) => fin_index,
        None => return Err("ERROR: clean_sender_id end".to_string()),
    };
    if debut > fin {
        Err("ERROR: clean_sender_id matrix-commander output unreadable".to_string())
    } else {
        let clean_sender_id = &raw_sender_id[debut..fin];
        Ok(clean_sender_id.to_string())
    }
}

pub fn clean_sender_name(raw_sender_name:String) -> Result<String, String> {
    let debut = match raw_sender_name.find("sender") {
        Some(debut_index) => debut_index + 7,
        None => return Err("ERROR: clean_sender_name start".to_string()),
    };
    let fin = match raw_sender_name.find("[") {
        Some(fin_index) => fin_index - 1,
        None => return Err("ERROR: clean_sender_name end".to_string()),
    };
    if debut > fin {
        Err("clean_sender_name ERROR: Matrix-Commander output unreadable".to_string())
    } else {
        let raw_sender_name = &raw_sender_name[debut..fin];
        Ok(raw_sender_name.to_string())
    }
}
