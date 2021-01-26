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
    // _initialisation de la liste des mots trigger "trigger_word_list": qui déclenchent une réponse de botbot
    // _la liste est placée dans un tableau remplis depuis la db pour pas à avoir à faire une requête
    // dans la db à chaque fois que botbot doit analyser les phrases.
    let (connection_db_result, mut trigger_word_list) = init_db ();

    // _controle de la connexion à la db
    // _si error on quite le programme
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
    // _si error on quite le programme
    let mut matrix_commander =
        match matrix_commander_daemon_launch() {
            Ok(matrix_commander_ctrl) => matrix_commander_ctrl,
            Err(e) => {
                println!("!!! Fail to lauch matrix-commander: {}", e);
                return
            }
        };

    // _crée une object 'processus" que l'on va pouvoir interroger pour vérifier que matrix-commander est toujours en vie
    // _si error on quite le programme
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

    // _
    // _si error on quite le programme
    let matrix_commander_raw_buffer =
        match matrix_commander.stdout.as_mut(){
            Some(matrix_commander_raw_buffer) => matrix_commander_raw_buffer,
            None => {
                println!("!!! fail to attach buffer");
                return
            }
        };

    // _crée un buffer allimenter par le stdout du processus matrix-commander
    let mut matrix_commander_ready_buffer = BufReader::new(matrix_commander_raw_buffer);

    // _crée la variable "line_from_buffer" qui va pouvoir réceptionner les data du buffer ligne à ligne
    let mut line_from_buffer = String::new();

    // _pré-construit le regex pour identifier les numéros de tickets
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
        // _split la ligne de buffer selon le char "|" cf: https://github.com/8go/matrix-commander
        let raw_data: Vec<&str> = line_from_buffer.split('|').collect();
        // _check que la trame a bien 5 partie cf: https://github.com/8go/matrix-commander
        if raw_data.len() == 4 {
            // _on crée la variable raw_message qui est la dernière partie de la trame
            // _on mets tout en lowercase + on retire les accents afin de maximiser les match dans la db
            let mut raw_message = String::from(raw_data[3]);
            raw_message.make_ascii_lowercase();
            // _on ignore les trames qui commencent par '>' qui sont dans matrix la reprise d'un message auquel on répond
            let raw_message_fist_char = raw_message.chars().nth(1).unwrap_or(' ');
            if raw_message_fist_char !=  '>' {
                // _controle et clean des 5 trames et création des 5 variables pour créer un objet Message
                // _si error: vide buffer + sortie de loop
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
                    // _création d'un Message
                    let incoming_message = Message{room_origin: clean_room, room_id: clean_room_id, sender_id: clean_sender_id, sender_name: clean_sender_name, m_message: clean_message};
                    // _retour de la réponse en fonction du global trigger (botbot || #ticket) dans raw_message via la methode .thinking
                    let trigger_answer_result =
                        if raw_message.contains("botbot") {
                            println!("PLOP");
                                let thinking_check =
                                    match incoming_message.thinking(&adminsys_list, &admincore_list, &mut trigger_word_list, &connection_db){
                                        Ok(answer_ctrl) => Ok(answer_ctrl),
                                        Err(e) => Err(format!("ERROR botbot thinking: {}", e)),
                                    };
                                thinking_check
                        } else if ticket_re.is_match(&raw_message)  && incoming_message.room_origin == "fdn-tickets-internal" {
                                //_isole le numéro du ticket avec le regex
                                let regex_capture = ticket_re.captures(&incoming_message.m_message).unwrap();
                                let raw_ticket_number = match regex_capture.get(0) {
                                    Some(raw_ticket_number_ctrl) => raw_ticket_number_ctrl.as_str(),
                                    None => continue,
                                };
                                let ticket_number = raw_ticket_number.to_string();
                                let ticket_check=
                                match incoming_message.ticket(ticket_number){
                                    Ok(answer_ctrl) => Ok(answer_ctrl),
                                    Err(e) => Err(format!("ERROR ticket: {}", e)),
                                };
                                ticket_check
                        } else {
                                Err(format!("No global trigger found"))
                        };
                    // _controle du résultat de .thinking si ok affichage de la réponse en console
                    // _si error: vide buffer + sortie de loop
                    let trigger_answer =
                        match trigger_answer_result {
                            Ok(trigger_answer_result_ctrl) => {
                                println!("Botbot {}", trigger_answer_result_ctrl);
                                trigger_answer_result_ctrl
                            }
                            Err(e) =>  {
                                println!("Error: {}", e);
                                line_from_buffer.clear();
                                continue
                            }
                        };
                    // _affichage de la réponse dans la room
                    // _si error: vide buffer + sortie de loop
                    let _talking_status =
                        match incoming_message.talking(trigger_answer){
                            Ok(talking_status_ctrl) => talking_status_ctrl.id(),
                            Err(e) =>  {
                                println!("Error: {}", e);
                                line_from_buffer.clear();
                                continue
                            }
                        };
                }
            }
            // _vide le du buffer à chaque boucle
            line_from_buffer.clear();
        }
}
