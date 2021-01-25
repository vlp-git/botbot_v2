use std::process::{Command, Stdio, Child};
use std::io::Error;
pub use matrix_mgmt::*;
pub mod matrix_mgmt;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION de lancement du processus matrix_commander

// _initialise le daemon matrix-commander
pub fn matrix_commander_daemon_launch() -> Result<Child, Error> {
    let daemon = Command::new("./../matrix-commander/matrix-commander.py")
        .arg("-c./../matrix-commander/credentials.json")
        .arg("-s./../matrix-commander/store/")
        .arg("-lforever")
        .stdout(Stdio::piped())
        .spawn();
    daemon
}
