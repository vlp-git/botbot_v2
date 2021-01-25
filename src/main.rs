////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  botbot v2 by vlp

use std::io::{BufRead, BufReader};
use procfs::process::Process;
use regex::Regex;
mod message;
use crate::message::*;
mod database;
use crate::database::*;
mod matrix;
use crate::matrix::*;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION principale

fn main() {

    println!("///// botbot v2 by lovely fdn team");

    // _initialisation de la liste des mots trigger: qui déclenchent une réponse de botbot
    // _la liste est placée dans un tableau remplis depuis la db pour pas à avoir à faire une requête
    // dans la db à chaque fois que botbot doit analyser les phrases.
    //let mut trigger_word_list: Vec<String> = Vec::new();

    // _liste des admins ayant accès au mode admin de botbot
    let mut adminsys_list: Vec<String> = Vec::new();
    let mut admincore_list: Vec<String> = Vec::new();

    // _team ADMINCORE
    admincore_list.push("@belette:uc.neviani.fr".to_string());
    admincore_list.push("@afriqs:matrix.fdn.fr".to_string());
    admincore_list.push("@asmadeus:codewreck.org".to_string());
    admincore_list.push("@tom28:matrix.fdn.fr".to_string());

    // _team ADMINSYS
    adminsys_list.push("@vlp:matrix.fdn.fr".to_string());
    adminsys_list.push("@belette:uc.neviani.fr".to_string());
    adminsys_list.push("@afriqs:matrix.fdn.fr".to_string());
    adminsys_list.push("@asmadeus:codewreck.org".to_string());
    adminsys_list.push("@tom28:matrix.fdn.fr".to_string());
    adminsys_list.push("@khrys:matrix.fdn.fr".to_string());
    adminsys_list.push("@olb:matrix.org".to_string());
    adminsys_list.push("@vg:matrix.fdn.fr".to_string());
    adminsys_list.push("@blackmoor:matrix.fdn.fr".to_string());
    adminsys_list.push("@dino:matrix.fdn.fr".to_string());
    adminsys_list.push("@hamo:matrix.fdn.fr".to_string());
    adminsys_list.push("@stephaneascoet:matrix.fdn.fr".to_string());
    adminsys_list.push("@symeon:matrix.fdn.fr".to_string());
    adminsys_list.push("@youpi:matrix.fdn.fr".to_string());
    adminsys_list.push("@mlrx:matrix.fdn.fr".to_string());

    println!("[Database]");

    // _connexion à la db ou création de la db si n'existe pas
    let (connection_db_result, mut trigger_word_list) = init_db ();

    let connection_db =
        match connection_db_result {
            Ok(connection_db_ctrl) => {
                println!(" > Database initialized with {} words", trigger_word_list.len());
                connection_db_ctrl
            }
            Err(e) => {
                println!("!!! Database initialization failed: {}", e);
                return
            }
        };

    println!("[Matrix Connection]");

    // _créer un processus fils au programme qui lance matrix-commander et qui pipe son flux stdout
    let mut matrix_commander =
        match matrix_commander_daemon_launch() {
            Ok(matrix_commander_ctrl) => matrix_commander_ctrl,
            Err(e) => {
                println!("!!! Fail to lauch matrix-commander: {}", e);
                return
            }
        };

    // _crée une object 'processus" que l'on va pouvoir interroger pour vérifier que matrix-commander est toujours en vie
    let matrix_pid =
        match Process::new(matrix_commander.id() as i32) {
            Ok(matrix_pid_ctrl) => {
                println!(" > matrix-commander launched: pid {}", matrix_pid_ctrl.pid);
                matrix_pid_ctrl
            }
            Err(e) => {
                println!("!!! fail to get matrix-commander pid: {}", e);
                return
            }
        };

    let matrix_commander_raw_buffer =
        match matrix_commander.stdout.as_mut(){
            Some(matrix_commander_raw_buffer) => matrix_commander_raw_buffer,
            None => {
                println!("!!! fail to attach buffer");
                return
            }
        };

    let mut matrix_commander_ready_buffer = BufReader::new(matrix_commander_raw_buffer);

    let mut line_from_buffer = String::new();

    let ticket_to_search_re = "#[0-9]{4,6}".to_string();
    let ticket_re =
        match Regex::new(&ticket_to_search_re){
            Ok(ticket_re_ctrl) => ticket_re_ctrl,
            Err(_e) => {
                println!("!!! fail to build ticket regex");
                return
            }
        };

    println!("[botbot is running]");

    // _boucle global qui est bloquante à cause de read.line qui attend un '\n' pour avancer
    loop {

        // _vérifie que le 'processus' de matrix-commander existe toujours en mémoire sinon arréte le program
        if matrix_pid.statm().unwrap().size == 0 {
            println!("matrix-commander do not respond, the application will shutdown");
            return;
        }

        // _lecture ligne à ligne du buffer
        let _buffer_control =
            match matrix_commander_ready_buffer.read_line(&mut line_from_buffer) {
                Ok(_buffer_control_ctrl) => _buffer_control_ctrl,
                Err(e) => {
                    println!("Unreadable line: {}", e);
                    line_from_buffer.clear();
                    continue;
                }
            };

        // _check que la trame dans la 1ère ligne du buffer corresponde bien à une entrée correcte de matrix-commander: https://github.com/8go/matrix-commander
        // _càd: trame de 4 parties séparées par des |
        let raw_data: Vec<&str> = line_from_buffer.split('|').collect();
        if raw_data.len() == 4 {
            // _check du mot clef botbot peu importe la casse mais vérifie que botbot ne soit pas juste dans le reply
            let mut trigger = String::from(raw_data[3]);
            trigger.make_ascii_lowercase();

            // _on ignore les reply qui commencent par '>'
            let reply_check = trigger.chars().nth(1).unwrap_or(' ');
            if trigger.contains("botbot") && reply_check !=  '>' {
                let (clean_room_id, clean_room, clean_sender_id, clean_sender_name, clean_message) =
                    match clean_trame(raw_data){
                        Ok(trame_ctrl) => {
                            trame_ctrl
                        }
                        Err(_e) => {
                            line_from_buffer.clear();
                            continue
                        },
                    };
                    let incoming_message = Message{_room_origin: clean_room, room_id: clean_room_id, sender_id: clean_sender_id, sender_name: clean_sender_name, m_message: clean_message};
                    let _talking_check =
                        match incoming_message.thinking(&adminsys_list, &admincore_list, &mut trigger_word_list, &connection_db){
                            Ok(answer_ctrl) => {
                                println!("botbot: {}", answer_ctrl);
                                let _talking_status =
                                    match incoming_message.talking(answer_ctrl){
                                        Ok(talking_child) => {
                                            Ok(talking_child.id())
                                        }
                                        Err(e) => {
                                            println!("ERROR: talking - {}", e);
                                            Err(format!("ERROR talking: {}", e))
                                        },
                                    };
                            }
                            Err(e) => {
                                println!("ERROR: thinking - {}", e);
                                line_from_buffer.clear();
                                continue
                            }
                        };
            }
            else if ticket_re.is_match(&trigger) && reply_check !=  '>' {
                let (clean_room_id, clean_room, clean_sender_id, clean_sender_name, clean_message) =
                    match clean_trame(raw_data){
                        Ok(trame_ctrl) => {
                            trame_ctrl
                        }
                        Err(_e) => {
                            line_from_buffer.clear();
                            continue
                        },
                    };
                if clean_room == "fdn-tickets-internal" {
                let caps = ticket_re.captures(&clean_message).unwrap();
                let clean_caps = match caps.get(0) {
                    Some(clean_caps_ctrl) => clean_caps_ctrl.as_str(),
                    None => continue,
                };
                let clean_message = clean_caps.to_string();
                let incoming_message = Message{_room_origin: clean_room, room_id: clean_room_id, sender_id: clean_sender_id, sender_name: clean_sender_name, m_message: clean_message};
                let _ticket_check=
                    match incoming_message.ticket(){
                        Ok(answer_ctrl) => {
                            println!("{}", answer_ctrl);
                            let _talking_status =
                                match incoming_message.talking(answer_ctrl){
                                    Ok(talking_child) => {
                                        Ok(talking_child.id())
                                    }
                                    Err(e) => {
                                        Err(format!("ERROR ticket talking: {}", e))
                                    },
                                };
                        }
                        Err(e) => {
                            println!("ERROR: ticket - {}", e);
                            line_from_buffer.clear();
                            continue
                        }
                    };
                }
            }
        }
        // _vide la zone de lecture du buffer à chaque boucle
        line_from_buffer.clear();
    }
}
