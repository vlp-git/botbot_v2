////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  botbot v2 by vlp

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio, Child};
use sqlite::{Connection, State};
use unidecode::unidecode;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  Structure et traits des messages reçus

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
    fn thinking(&self, trigger_word_list: &mut Vec<String>, connection_db: &Connection) -> String {
        let mut choice = String::from(unidecode(&self.m_message).to_string());
        choice.make_ascii_lowercase();
        if choice.contains("admin add") && &self.sender_id == "@vlp:matrix.fdn.fr" {
            let debut_trigger = choice.find("[").unwrap() + 1;
            let fin_trigger = choice.find("]").unwrap();
            let trigger_to_add = &choice[debut_trigger..fin_trigger];
            let debut_answer = choice.rfind("[").unwrap() + 1;
            let fin_answer = choice.rfind("]").unwrap();
            let answer_to_add = &choice[debut_answer..fin_answer];
            let mut insert_statement =
                match connection_db.prepare("INSERT INTO talking (trigger, answer) VALUES (?, ?);"){
                    Ok(add_chat) => {
                        add_chat
                    }
                    Err(e) => {
                        println!("Error add word: {}", e);
                        return "ERROR".to_string()
                        }
                  };
                insert_statement.bind(1, trigger_to_add).unwrap();
                insert_statement.bind(2, answer_to_add).unwrap();
                insert_statement.next().unwrap();
                trigger_word_list.push(trigger_to_add.to_string());
        } else if choice.contains("admin del") && &self.sender_id == "@vlp:matrix.fdn.fr"{
            let debut_trigger = choice.find("[").unwrap() + 1;
            let fin_trigger = choice.find("]").unwrap();
            let trigger_to_del = &choice[debut_trigger..fin_trigger];
            let mut del_statement =
                match connection_db.prepare("DELETE FROM talking WHERE trigger=?"){
                    Ok(check_chat) => {
                        check_chat
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        return "ERROR".to_string()
                        }
                  };
            del_statement.bind(1, trigger_to_del).unwrap();
            del_statement.next().unwrap();
            trigger_word_list.retain(|x| *x != trigger_to_del);
        } else{
            for x in trigger_word_list {
                if choice.contains(&x[..]) {
                    let mut select_statement =
                        match connection_db.prepare("SELECT answer FROM talking where trigger=?"){
                            Ok(match_word) => {
                                match_word
                            }
                            Err(e) => {
                                println!("Error select word: {}", e);
                                return "ERROR".to_string()
                                }
                          };
                    select_statement.bind(1, &x[..]).unwrap();
                    while let State::Row = select_statement.next().unwrap() {
                        let blabla = select_statement.read::<String>(0).unwrap();
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
    let debut = raw_room_origin.find("room").unwrap_or(0) + 5;
    let fin = raw_room_origin.find("[").unwrap_or(0) - 1;
    let clean_room_origin = &raw_room_origin[debut..fin];
    if debut == 0 || fin == 0 {
        return "ERROR".to_string()
    }
    else{
        return clean_room_origin.to_string();
    }
}

fn clean_room_id(raw_room_id:String) -> String {
    let debut = raw_room_id.find("[").unwrap_or(0) + 1;
    let fin = raw_room_id.find("]").unwrap_or(0);
    let clean_room_id = &raw_room_id[debut..fin];
    if debut == 0 || fin == 0 {
        return "ERROR".to_string()
    }
    else{
        return clean_room_id.to_string();
    }
}

fn clean_sender_id(raw_sender_id:String) -> String {
    let debut = raw_sender_id.find("[").unwrap_or(0) + 1;
    let fin = raw_sender_id.find("]").unwrap_or(0);
    let clean_sender_id = &raw_sender_id[debut..fin];
    if debut == 0 || fin == 0 {
        return "ERROR".to_string()
    }
    else{
        return clean_sender_id.to_string();
    }
}

fn clean_sender_name(raw_sender_name:String) -> String {
    let debut = raw_sender_name.find("sender").unwrap_or(0) + 7;
    let fin = raw_sender_name.find("[").unwrap_or(0) - 1;
    let clean_sender_name = &raw_sender_name[debut..fin];
    if debut == 0 || fin == 0 {
        return "ERROR".to_string()
    }
    else{
        return clean_sender_name.to_string();
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION initialisation de la db

fn init_db(connection_db: &Connection, trigger_word_list: &mut Vec<String>) -> bool {

    let mut create_table_statement =
        match connection_db.prepare("CREATE TABLE if not exists talking (chat_id INTEGER PRIMARY KEY, trigger TEXT not null UNIQUE, answer TEXT not null);") {
            Ok(create_table) => {
                println!(" > Talking table prepared");
                create_table
            }
            Err(e) => {
                println!(" > Fail to prepare talking table: {}", e);
                return false;
                }
          };

    while let State::Row = create_table_statement.next().unwrap() {}

    let mut add_words_statement =
        match connection_db.prepare("SELECT trigger FROM talking") {
            Ok(fill_list) => {
                println!(" > Wordlist loaded");
                fill_list
            }
            Err(e) => {
                println!(" > Fail to load wordlist.db: {}", e);
                return false;
                }
          };

    while let State::Row = add_words_statement.next().unwrap() {
            let word_to_add = add_words_statement.read::<String>(0).unwrap();
            println!("'{}' will be added as trigger word", word_to_add);
            trigger_word_list.push(word_to_add);
        }

    return true;
}

fn matrix_commander_daemon_launch() -> Child {
    let daemon = Command::new("/home/vlp/git/matrix-commander/matrix-commander.py")
        .arg("-c/home/vlp/git/matrix-commander/credentials.json")
        .arg("-s/home/vlp/git/matrix-commander/store/")
        .arg("-lforever")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
return daemon
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION principale

fn main() {

    // _initialisation de la liste des mots trigger: qui déclenchent une réponse de botbot
    // _la liste est placée dans un tableau remplis depuis la db pour pas à avoir à faire une requête
    // dans la db à chaque fois que botbot doit analyser les phrases.
    let mut trigger_word_list: Vec<String> = Vec::new();

    // _connexion à la db ou création de la db si n'existe pas
    let connection_db =
        match Connection::open("worterkasten.db") {
            Ok(db) => {
                println!("Database opened or created !");
                db
            }
            Err(e) => {
                println!("Error opening database: {}", e);
                return;
            }
         };

    // _initialisation de la db
    match init_db(&connection_db, &mut trigger_word_list) {
        true => println!("Database initialized"),
        false => {
            println!("Database initialization failed !");
            return
        }
    };

    // _créer un processus fils au programme qui lance matrix-commander et qui pipe son flux stdout
    let mut matrix_commander = matrix_commander_daemon_launch();
    let mut matrix_commander_stdout_buffer = BufReader::new(matrix_commander.stdout.as_mut().unwrap());

    let mut line = String::new();

     loop {
         // lit le buffer ligne à ligne
         let _len_line = matrix_commander_stdout_buffer.read_line(&mut line).unwrap();
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
                 incoming_message.m_answer = incoming_message.thinking(&mut trigger_word_list, &connection_db);
                 if incoming_message.m_answer != "ERROR".to_string() {
                     incoming_message.talking();
                 }
            }
        }
        line.clear();
    }

}
