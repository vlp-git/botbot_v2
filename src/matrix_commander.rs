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
