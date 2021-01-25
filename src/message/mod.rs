use unidecode::unidecode;
use sqlite::Connection;
use std::process::{Command, Child};
pub use message_mgmt::*;
pub mod message_mgmt;
use crate::database::*;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  Structure et traits des messages reçus

// _structure d'un Message
pub struct Message{
    pub _room_origin: String,
    pub room_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub m_message: String,
}

// _traits de Message
impl Message{
    // _détermine les actions de botbot lorsqu'il est déclenché
    pub fn thinking(&self, adminsys_list: &Vec<String>, admincore_list: &Vec<String>, trigger_word_list: &mut Vec<String>, connection_db: &Connection) -> Result<String, String> {
        let choice = String::from(unidecode(&self.m_message).to_string());
        let mut botbot_phrase = String::from(unidecode(&self.m_message).to_string());
        // _uppercases
        botbot_phrase.make_ascii_lowercase();
        // _mode admin
        let answer =
            if botbot_phrase.contains("botbot admin") && adminsys_list.contains(&self.sender_id) {
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
                            match get_left_arg(&choice) {
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
                let mut iterator = adminsys_list.iter();
                let mut liste_to_ping = String::from("ping: ");
                while let Some(x) = iterator.next() {
                    let fin_mark =
                        match x.find(":") {
                            Some(fin_mark_index) => fin_mark_index,
                            None => continue,
                        };
                    liste_to_ping += &x[..fin_mark];
                    liste_to_ping += ", ";
                }
                let chat_to_ping = format!("Hello les adminsys: {} vous contacte ! {}", &self.sender_name, &liste_to_ping[0..liste_to_ping.len()-2]);
                Ok(chat_to_ping)
            } else if botbot_phrase.contains("ping admincore") {
                let mut iterator = admincore_list.iter();
                let mut liste_to_ping = String::from("ping: ");
                while let Some(x) = iterator.next() {
                    let fin_mark =
                        match x.find(":") {
                            Some(fin_mark_index) => fin_mark_index,
                            None => continue,
                        };
                    liste_to_ping += &x[..fin_mark];
                    liste_to_ping += ", ";
                }
                let chat_to_ping = format!("Hello les adminsys: {} vous contacte ! {}", &self.sender_name, &liste_to_ping[0..liste_to_ping.len()-2]);
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

    // _détermine les actions de botbot lorsqu'il voit un numéro de ticket
    pub fn ticket(&self) -> Result<String, String> {
        // _ajoute au numéro de ticket l'url de RT
        let ticket_url = format!("Ticket: https://tickets.fdn.fr/rt/Ticket/Display.html?id={}", &self.m_message[1..]);
        Ok(ticket_url)
    }

    // _fait parler botbot
    pub fn talking(&self, phrase_to_say: String) -> Result<Child, String> {
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
