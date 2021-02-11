use std::process::{Command, Stdio, Child};
use std::io::Error;
pub use matrix_mgmt::*;
pub mod matrix_mgmt;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION de lancement du processus matrix_commander

// _initialise le daemon matrix-commander
pub fn matrix_commander_daemon_launch() -> Result<Child, Error> {
    let daemon = Command::new(crate::MATRIX_FOLDER)
        .arg(crate::MATRIX_CREDITENTIALS)
        .arg(crate::MATRIX_DB_FOLDER)
        .arg("-lforever")
        .arg("--log-level")
        .arg("ERROR")
        .stdout(Stdio::piped())
        .spawn();
    daemon
}

// _envoie un message
pub fn matrix_commander_message_send(room: String, blabla: String) -> Result<Child, String> {
    let message_to_send =
        match Command::new(crate::MATRIX_FOLDER)
        .arg(crate::MATRIX_CREDITENTIALS)
        .arg(crate::MATRIX_DB_FOLDER)
        .arg(room)
        .arg(blabla)
        .arg("--log-level")
        .arg("ERROR")
        .spawn() {
            Ok(talking_status_ctrl) => Ok(talking_status_ctrl),
            Err(e) => Err(format!("ERROR: sending message - {}", e)),
        };
    message_to_send
}
