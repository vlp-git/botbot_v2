////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  botbot v2 by vlp

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use sqlite::{Connection, State};
use unidecode::unidecode;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  Structure et traits

struct Message{
    // structure d'un message reçu
    _room_origin: String,
    room_id: String,
    sender_id: String,
    _sender_name: String,
    m_message: String,
    m_answer: String,
}

impl Message{
    fn _print_full_message(&self){
        println!("{}", self._room_origin);
        println!("{}", self.room_id);
        println!("{}", self.sender_id);
        println!("{}", self._sender_name);
        println!("{}", self.m_message);
        println!("{}", self.m_answer);
    }
    fn thinking(&self, word_list: &Vec<String>, connection_db: &Connection) -> String {
        let mut choice = String::from(unidecode(&self.m_message).to_string());
        choice.make_ascii_lowercase();
        if choice.contains("admin add") && &self.sender_id == "@vlp:matrix.fdn.fr" {
            println!("admin mode: ON");
            let debut_trigger = choice.find("[").unwrap() + 1;
            let fin_trigger = choice.find("]").unwrap();
            let _trigger_to_add = &choice[debut_trigger..fin_trigger];
            let debut_answer = choice.rfind("[").unwrap() + 1;
            let fin_answer = choice.rfind("]").unwrap();
            let _answer_to_add = &choice[debut_answer..fin_answer];
            let mut _insert_statement =
                match connection_db.execute("INSERT INTO talking (trigger, answer) VALUES ('test3', 'super test3');"){
                    Ok(add_chat) => {
                        add_chat
                    }
                    Err(e) => {
                        println!("Error add word: {}", e);
                        return "ERROR".to_string()
                        }
                  };
        } else{
            for x in word_list {
                if choice.contains(x) {
                    let mut statement =
                        match connection_db.prepare("SELECT answer FROM talking where trigger=?"){
                            Ok(match_word) => {
                                match_word
                            }
                            Err(e) => {
                                println!("Error select word: {}", e);
                                return "ERROR".to_string()
                                }
                          };
                    statement.bind(1, &x[..]).unwrap();
                    while let State::Row = statement.next().unwrap() {
                        let blabla = statement.read::<String>(0).unwrap();
                        return blabla;
                    }
                    break;
                }
            }
        }
        return "ERROR".to_string()
    }
    fn talking(&self){
        let mut blabla = "-m".to_string();
        blabla.push_str(&self.m_answer);
        let mut room = "-r".to_string();
        room.push_str(&self.room_id);
        Command::new("/home/vlp/git/matrix-commander/matrix-commander.py")
            .arg("-c/home/vlp/git/matrix-commander/credentials.json")
            .arg("-s/home/vlp/git/matrix-commander/store/")
            .arg(room)
            .arg(blabla)
            .spawn()
            .expect("message failed");
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTIONS pour nettoyer les trames de matrix-commander

fn clean_room_origin(raw_room_origin:String) -> String {
    let debut = raw_room_origin.find("room").unwrap() + 5;
    let fin = raw_room_origin.find("[").unwrap() - 1;
    let clean_room_origin = &raw_room_origin[debut..fin];
    return clean_room_origin.to_string();
}

fn clean_room_id(raw_room_id:String) -> String {
    let debut = raw_room_id.find("[").unwrap() + 1;
    let fin = raw_room_id.find("]").unwrap();
    let clean_room_id = &raw_room_id[debut..fin];
    return clean_room_id.to_string();
}

fn clean_sender_id(raw_sender_id:String) -> String {
    let debut = raw_sender_id.find("[").unwrap() + 1;
    let fin = raw_sender_id.find("]").unwrap();
    let clean_sender_id = &raw_sender_id[debut..fin];
    return clean_sender_id.to_string();
}

fn clean_sender_name(raw_sender_name:String) -> String {
    let debut = raw_sender_name.find("sender").unwrap() + 7;
    let fin = raw_sender_name.find("[").unwrap() - 1;
    let clean_sender_name = &raw_sender_name[debut..fin];
    return clean_sender_name.to_string();
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION principale

fn main() {

    let mut trigger_word_list: Vec<String> = Vec::new();

    let conn =
        match Connection::open("worterkasten.db") {
            Ok(db) => {
                println!("Worterkasten.db open !");
                db
            }
            Err(e) => {
                println!("Error opening worterkasten.db: {}", e);
                return;
            }
         };

    let mut statement =
        match conn.prepare("SELECT trigger FROM talking") {
            Ok(fill_list) => {
                println!("wordlist loaded");
                fill_list
            }
            Err(e) => {
                println!("Error loading wordlist.db: {}", e);
                return;
                }
          };

    while let State::Row = statement.next().unwrap() {
        let word_to_add = statement.read::<String>(0).unwrap();
        println!("'{}' will be added as trigger word", word_to_add);
        trigger_word_list.push(word_to_add);
        }

    //lance matrix-commander en background et pipe son stdout dans le programme
    let mut matrix_commander_shell = Command::new("/home/vlp/git/matrix-commander/matrix-commander.py")
        .arg("-c/home/vlp/git/matrix-commander/credentials.json")
        .arg("-s/home/vlp/git/matrix-commander/store/")
        .arg("-lforever")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

         let mut child_out = BufReader::new(matrix_commander_shell.stdout.as_mut().unwrap());
         let mut line = String::new();

     loop {
         // lit le buffer ligne à ligne
         let _len_line = child_out.read_line(&mut line).unwrap();
         // check que la trame dans la 1ère ligne du buffer corresponde bien à une entrée correcte de matrix-commander: https://github.com/8go/matrix-commander
         // càd: trame de 4 parties séparées par des |
         let raw_data: Vec<&str> = line.split('|').collect();
         if raw_data.len() == 4 {
             // check du mot clef botbot peu importe la casse mais vérifie que botbot ne soit pas juste dans le reply
             let mut trigger = String::from(raw_data[3]);
             trigger.make_ascii_lowercase();
             let reply_check = trigger.chars().nth(1).unwrap();
             if trigger.contains("botbot") && reply_check !=  '>' {
                 // construction du Message: cf la struct
                 let mut incoming_message = Message{_room_origin: clean_room_origin(String::from(raw_data[0])), room_id: clean_room_id(String::from(raw_data[0])), sender_id: clean_sender_id(String::from(raw_data[1])), _sender_name: clean_sender_name(String::from(raw_data[1])), m_message: String::from(raw_data[3]), m_answer: String::from("")};
                 incoming_message.m_answer = incoming_message.thinking(&trigger_word_list, &conn);
                 if incoming_message.m_answer != "ERROR".to_string() {
                     incoming_message.talking();
                 }
            }
        }
        line.clear();
    }

}
