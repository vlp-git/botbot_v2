////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  botbot v2 by vlp

use std::io::{BufRead, BufReader};
use unidecode::unidecode;
use procfs::process::Process;
use sqlite::Connection;
use std::process::{Command, Child};
pub use mgmt::*;
pub use matrix_commander::*;
pub use sqlite_db::*;
mod mgmt;
mod matrix_commander;
mod sqlite_db;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  Structure et traits des messages reçus

struct Message{
    // structure d'un message reçu
    _room_origin: String,
    room_id: String,
    sender_id: String,
    sender_name: String,
    m_message: String,
}

impl Message{
    // _fonction qui détermine les actions de botbot lorsqu'il est déclenché
    fn thinking(&self, admin_list: &Vec<String>, trigger_word_list: &mut Vec<String>, connection_db: &Connection) -> Result<String, String> {
        let choice = String::from(unidecode(&self.m_message).to_string());
        let mut botbot_phrase = String::from(unidecode(&self.m_message).to_string());
        // _uppercases
        botbot_phrase.make_ascii_lowercase();
        // _mode admin
        let answer =
            if botbot_phrase.contains("botbot admin") && admin_list.contains(&self.sender_id) {
                let admin_answer =
                    // _mode admin pour ajout de trigger
                    if botbot_phrase.contains("admin add") {
                        let chat_to_add =
                              match get_left_arg(&choice) {
                                  Ok(trigger_to_add_ctrl) => {
                                      let answer_to_add =
                                              match get_right_arg(&choice) {
                                                  Ok(answer_to_add_ctrl) => {
                                                      let process_to_add =
                                                          match add_chat(trigger_to_add_ctrl, answer_to_add_ctrl, connection_db, trigger_word_list) {
                                                              Ok(chat_to_add_ctrl) => Ok(format!("[admin mode by: {}] {} ajouté !", &self.sender_name, chat_to_add_ctrl)),
                                                              Err(e) => Err(format!("ERROR: chat_to_add process to add - {}", e)),
                                                          };
                                                      process_to_add
                                                  }
                                                  Err(e) => Err(format!("ERROR: chat_to_add get answer - {}", e)),
                                              };
                                      answer_to_add
                                  }
                                  Err(e) => Err(format!("ERROR: chat_to_add get trigger- {}", e)),
                              };
                        chat_to_add
                    // _mode admin pour suppression de trigger
                    } else if botbot_phrase.contains("admin del") {
                        let chat_to_del =
                            match mgmt::get_left_arg(&choice) {
                                Ok(trigger_to_del_ctrl) => {
                                    let proceed_to_del =
                                        match del_chat(trigger_to_del_ctrl, connection_db, trigger_word_list) {
                                            Ok(_chat_to_del_ctrl) => Ok(format!("[admin mode by: {}] {} supprimé !", &self.sender_name, _chat_to_del_ctrl)),
                                            Err(e) => Err(format!("ERROR: chat_to_del proceed to del - {}", e)),
                                        };
                                        proceed_to_del
                                }
                                Err(e) => Err(format!("ERROR: chat_to_del match trigger - {}", e)),
                            };
                        chat_to_del
                    // _fail de commande admin
                    } else if botbot_phrase.contains("admin alert") {
                        Ok("plop".to_string())
                    } else {
                        Err("ERROR: no admin command".to_string())
                    };
                admin_answer
            } else if botbot_phrase.contains("ping adminsys") {
                let mut iterator = admin_list.iter();
                let mut liste_to_ping = String::from("ping: ");
                while let Some(x) = iterator.next() {
                    let debut_mark =
                        match x.find("@") {
                            Some(debut_mark_index) => debut_mark_index + 1,
                            None => continue,
                    };
                    let fin_mark =
                        match x.find(":") {
                            Some(fin_mark_index) => fin_mark_index,
                            None => continue,
                        };
                    liste_to_ping += &x[debut_mark..fin_mark];
                    liste_to_ping += ", ";
                }
                let chat_to_ping = format!("Hello les adminsys: {} vous contact ! {}", &self.sender_name, &liste_to_ping[0..liste_to_ping.len()-2]);
                Ok(chat_to_ping)
            } else {
                // _réponse de botbot
                let chat_answer =
                    match get_answer(botbot_phrase, connection_db, trigger_word_list){
                        Ok(answer_ctrl) => {
                            // _remplace les %s par le nom du sender
                            let answer_with_name= &answer_ctrl[..].replace("%s", &self.sender_name);
                            // _remplace les %n par un retour à la ligne
                            let answer_with_new_line = &answer_with_name[..].replace("%n", "\n");
                            Ok(answer_with_new_line.to_string())
                        }
                        Err(e) => Err(format!("ERROR: return answer - {}",  e)),
                    };
                chat_answer
            };
        answer
    }
    fn talking(&self, phrase_to_say: String) -> Result<Child, String> {
        let mut blabla = "-m".to_string();
        blabla.push_str(&phrase_to_say[..]);
        let mut room = "-r".to_string();
        room.push_str(&self.room_id);
        let talking_status =
            match Command::new("./../matrix-commander/matrix-commander.py")
            .arg("-c./../matrix-commander/credentials.json")
            .arg("-s./..//matrix-commander/store/")
            .arg(room)
            .arg(blabla)
            .spawn() {
                Ok(talking_status_ctrl) => Ok(talking_status_ctrl),
                Err(e) => Err(format!("ERROR: sending message - {}", e)),
            };
        talking_status
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION principale

fn main() {

    println!("///// botbot v2 by lovely fdn team");

    // _initialisation de la liste des mots trigger: qui déclenchent une réponse de botbot
    // _la liste est placée dans un tableau remplis depuis la db pour pas à avoir à faire une requête
    // dans la db à chaque fois que botbot doit analyser les phrases.
    let mut trigger_word_list: Vec<String> = Vec::new();

    // _liste des admins ayant accès au mode admin de botbot
    let mut admin_list: Vec<String> = Vec::new();

    // _team ADMINSYS
    admin_list.push("@vlp:matrix.fdn.fr".to_string());
    admin_list.push("@belette:uc.neviani.fr".to_string());
    admin_list.push("@afriqs:matrix.fdn.fr".to_string());
    admin_list.push("@asmadeus:codewreck.org".to_string());
    admin_list.push("@tom28:matrix.fdn.fr".to_string());
    admin_list.push("@khrys:matrix.fdn.fr".to_string());

    println!("[Database]");

    // _connexion à la db ou création de la db si n'existe pas
    // let connection_db =
    //     match init_db (&mut trigger_word_list){
    //         Ok(connection_db_ctrl) => connection_db_ctrl,
    //         Err(e) => {
    //             println!("!!! Error opening database: {}", e);
    //             return
    //         }
    //     };


    // _connexion à la db ou création de la db si n'existe pas
    let connection_db =
        match Connection::open("worterkasten.db") {
            Ok(connection_db_ctrl) => {
                println!(" > Database opened");
                    match init_db_connection(&connection_db_ctrl, &mut trigger_word_list) {
                        Ok(init_db_ctrl) => {
                            println!(" > Database initialized with {} words", init_db_ctrl);
                        }
                        Err(e) => {
                            println!("!!! Database initialization failed: {}", e);
                            return
                        }
                    };
                connection_db_ctrl
            }
            Err(e) => {
                println!("!!! Error opening database: {}", e);
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

                // _construction du message: cf la struct
                let clean_room           =
                    match clean_room_origin(String::from(raw_data[0])) {
                        Ok(clean_room_ok) => clean_room_ok,
                        Err(_e) => {
                            line_from_buffer.clear();
                            continue
                        }
                    };
                let clean_room_id           =
                    match clean_room_id(String::from(raw_data[0])) {
                        Ok(clean_room_id_ok) => clean_room_id_ok,
                        Err(_e) => {
                            line_from_buffer.clear();
                            continue
                        }
                    };
                let clean_sender_id           =
                    match clean_sender_id(String::from(raw_data[1])) {
                        Ok(clean_sender_id_ok) => clean_sender_id_ok,
                        Err(_e) => {
                            line_from_buffer.clear();
                            continue
                        }
                    };
                let clean_sender_name           =
                    match clean_sender_name(String::from(raw_data[1])) {
                        Ok(clean_sender_name_ok) => clean_sender_name_ok,
                        Err(_e) => {
                            line_from_buffer.clear();
                            continue
                        }
                    };
                let clean_message = String::from(raw_data[3]);
                let incoming_message = Message{_room_origin: clean_room, room_id: clean_room_id, sender_id: clean_sender_id, sender_name: clean_sender_name, m_message: clean_message};
                match incoming_message.thinking(&admin_list, &mut trigger_word_list, &connection_db){
                    Ok(answer_ctrl) => {
                        println!("botbot: {}", answer_ctrl);
                        let _talking_status =
                            match incoming_message.talking(answer_ctrl){
                                Ok(talking_child) => {
                                    Ok(talking_child.id())
                                }
                                Err(e) => {
                                    Err(format!("ERROR talking: {}", e))
                                },
                            };
                    }
                    Err(e) => {
                        println!("ERROR: thinking - {}", e);
                        line_from_buffer.clear();
                        continue
                    }
                }
            }
        }
        // _vide la zone de lecture du buffer à chaque boucle
        line_from_buffer.clear();
    }
}
