////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  botbot v2 by vlp

use std::io::{BufRead, BufReader, Error};
use std::process::{Command, Stdio, Child};
use sqlite::{Connection, State};
use unidecode::unidecode;
use procfs::process::Process;
use rand::Rng;

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
    fn _print_full_message(&self){
        println!("{}", self._room_origin);
        println!("{}", self.room_id);
        println!("{}", self.sender_id);
        println!("{}", self.sender_name);
        println!("{}", self.m_message);
    }

    // _fonction qui détermine les actions de botbot lorsqu'il est déclenché
    fn thinking(&self, trigger_word_list: &mut Vec<String>, connection_db: &Connection) -> Result<String, String> {
        let choice = String::from(unidecode(&self.m_message).to_string());
        let mut botbot_phrase = String::from(unidecode(&self.m_message).to_string());
        ///// to split uppercases
        botbot_phrase.make_ascii_lowercase();
        // _mode admin pour ajout de trigger
        if botbot_phrase.contains("admin add") && &self.sender_id == "@vlp:matrix.fdn.fr" {
            let trigger_to_add =
                match get_left_arg(&choice) {
                    Ok(trigger_to_add_ctrl) => trigger_to_add_ctrl,
                    Err(e) => return Err(format!("ERROR: trigger_to_add - {}", e)),
                };
            let answer_to_add =
                match get_right_arg(&choice) {
                    Ok(answer_to_add_ctrl) => answer_to_add_ctrl,
                    Err(e) => return Err(format!("ERROR: answer_to_add - {}", e)),
                };
            if trigger_to_add == answer_to_add {
                return Err("ERROR: 2 arg needed [trigger] [answer]".to_string());
            }
            let chat_to_add =
                match add_chat(trigger_to_add, answer_to_add, connection_db, trigger_word_list) {
                    Ok(chat_to_add_ctrl) => Ok(format!("[admin mode by: {}] {} ajouté !", &self.sender_name, chat_to_add_ctrl)),
                    Err(e) => Err(format!("ERROR: chat_to_add - {}", e)),
            };
            chat_to_add
        // _mode admin pour suppression de trigger
        } else if botbot_phrase.contains("admin del") && &self.sender_id == "@vlp:matrix.fdn.fr"{
            let trigger_to_del =
                match get_left_arg(&choice) {
                    Ok(trigger_to_del_ctrl) => trigger_to_del_ctrl,
                    Err(e) => return Err(format!("ERROR: trigger_to_del - {}", e)),
                };
            let chat_to_del =
                match del_chat(trigger_to_del, connection_db, trigger_word_list) {
                    Ok(_chat_to_del_ctrl) => Ok(format!("[admin mode by: {}] {} supprimé !", &self.sender_name, _chat_to_del_ctrl)),
                    Err(e) => Err(format!("ERROR: chat_to_del - {}", e)),
                };
            chat_to_del
        // _réponse de botbot
        } else{
            let answer =
                match return_answer(botbot_phrase, connection_db, trigger_word_list){
                    Ok(answer_ctrl) => {
                        // _remplace les %s par le nom du sender
                        let answer_with_name= &answer_ctrl[..].replace("%s", &self.sender_name);
                        // _remplace les %n par un retour à la ligne
                        let answer_with_new_line = &answer_with_name[..].replace("%n", "\n");
                        Ok(answer_with_new_line.to_string())
                    }
                    Err(e) => return Err(format!("ERROR: return answer - {}", e)),
                };
            answer
        }
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

fn add_chat(trigger: String, answer: String, connection_db: &Connection, trigger_word_list: &mut Vec<String>) -> Result<String, String> {
    let mut insert_statement =
        match connection_db.prepare("INSERT INTO talking (trigger, answer) VALUES (?, ?);"){
            Ok(insert_statement_ctrl) => insert_statement_ctrl,
            Err(e) => return Err(format!("ERROR: add prepare db - {}", e)),
          };
        insert_statement.bind(1, &trigger[..]).unwrap();
        insert_statement.bind(2, &answer[..]).unwrap();
        let _run_statement =
            match insert_statement.next() {
                Ok(_run_statement_ctrl) => _run_statement_ctrl,
                Err(e) => return Err(format!("ERROR: process add trigger - {}", e)),
            };
        if !trigger_word_list.contains(&trigger.to_string()){
            trigger_word_list.push(trigger.to_string());
        }
        Ok(trigger)
}

fn del_chat(trigger: String, connection_db: &Connection, trigger_word_list: &mut Vec<String>) -> Result<String, String> {
    if !trigger_word_list.contains(&trigger) {
        return Err(format!("ERROR: trigger not in db"))
    }
    let mut del_statement =
        match connection_db.prepare("DELETE FROM talking WHERE trigger=?"){
            Ok(del_statement_ctrl) => del_statement_ctrl,
            Err(e) => return Err(format!("ERROR: del prepare db - {}", e)),
          };
    del_statement.bind(1, &trigger[..]).unwrap();
    let _run_statement =
        match del_statement.next() {
            Ok(_run_statement_ctrl) => _run_statement_ctrl,
            Err(e) => return Err(format!("ERROR: process del trigger - {}", e)),
        };
    trigger_word_list.retain(|x| *x != trigger);
    Ok(trigger)
}

fn return_answer(choice: String, connection_db: &Connection, trigger_word_list: &mut Vec<String>) -> Result<String, String> {
    let mut tmp_answers: Vec<String> = Vec::new();
    for x in trigger_word_list {
        if choice.contains(&x[..]) {
            let mut select_statement =
                match connection_db.prepare("SELECT answer FROM talking where trigger=?"){
                    Ok(select_statement_ctrl) => select_statement_ctrl,
                    Err(e) =>  return Err(format!("ERROR: select prepare db - {}", e)),
                  };
            select_statement.bind(1, &x[..]).unwrap();
            while let State::Row = select_statement.next().unwrap() {
                let blabla = select_statement.read::<String>(0).unwrap();
                tmp_answers.push(blabla);
            }
            continue;
        }
    }
    if tmp_answers.len() != 0 {
        let mut rng = rand::thread_rng();
        Ok(tmp_answers[rng.gen_range(0..tmp_answers.len())].to_string())
    }else{
        Err(format!("ERROR: no word found"))
    }
}
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTIONS pour nettoyer les trames de matrix-commander

fn clean_room_origin(raw_room_origin:String) -> Result<String, String> {
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

fn clean_room_id(raw_room_id:String) -> Result<String, String> {
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


fn clean_sender_id(raw_sender_id:String) -> Result<String, String> {
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

fn clean_sender_name(raw_sender_name:String) -> Result<String, String> {
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

fn get_left_arg(admin_msg: &String) -> Result<String, String> {
    let debut_mark =
        match admin_msg.find("[") {
            Some(debut_mark_index) => debut_mark_index + 1,
            None => return Err("ERROR: unable to find left arg start".to_string()),
        };
    let fin_mark =
        match admin_msg.find("]") {
            Some(fin_mark_index) => fin_mark_index,
            None => return Err("ERROR: unable to find left arg end".to_string()),
        };
    if debut_mark == fin_mark {
        Err("ERROR: no value in left arg".to_string())
    }
    else {
        Ok(admin_msg[debut_mark..fin_mark].to_string())
    }
}

fn get_right_arg(admin_msg: &String) -> Result<String, String> {
    let debut_mark =
        match admin_msg.rfind("[") {
            Some(debut_mark_index) => debut_mark_index + 1,
            None => return Err("ERROR: unable to find right arg start".to_string()),
        };
    let fin_mark =
        match admin_msg.rfind("]") {
            Some(fin_mark_index) => fin_mark_index,
            None => return Err("ERROR: unable to find right arg end".to_string()),
        };
    if debut_mark == fin_mark {
        Err("ERROR: no value in right arg".to_string())
    }
    else {
        Ok(admin_msg[debut_mark..fin_mark].to_string())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION initialisation de la db

fn init_db(connection_db: &Connection, trigger_word_list: &mut Vec<String>) -> Result<usize, String> {

    // _crée la table talking si elle n'existe pas
    let mut create_table_statement =
        match connection_db.prepare("CREATE TABLE if not exists talking (chat_id INTEGER PRIMARY KEY, trigger TEXT not null, answer TEXT not null);") {
            Ok(create_table_statement_ctrl) => create_table_statement_ctrl,
            Err(_e) => return Err("Talking table fail to initialized".to_string()),
          };

    while let State::Row = create_table_statement.next().unwrap() {}

    // _charge dans trigger_word_list tous les triggers de la table talking
    let mut add_words_statement =
        match connection_db.prepare("SELECT trigger FROM talking") {
            Ok(add_words_statement_ctrl) => add_words_statement_ctrl,
            Err(_e) => return Err("Fail to load wordlist.db".to_string()),
          };

    while let State::Row = add_words_statement.next().unwrap() {
            let word_to_add = add_words_statement.read::<String>(0).unwrap();
            if !trigger_word_list.contains(&word_to_add){
                trigger_word_list.push(word_to_add);
            }
        }

    Ok(trigger_word_list.len())
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION lancement du processus matrix_commander

fn matrix_commander_daemon_launch() -> Result<Child, Error> {
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
////////////////////////  FONCTION principale

fn main() {

    println!("///// botbot v2 by lovely fdn team");

    // _initialisation de la liste des mots trigger: qui déclenchent une réponse de botbot
    // _la liste est placée dans un tableau remplis depuis la db pour pas à avoir à faire une requête
    // dans la db à chaque fois que botbot doit analyser les phrases.
    let mut trigger_word_list: Vec<String> = Vec::new();

    // _liste des admins ayant accès au mode admin de botbot
    let _admin_list = ["@vlp:matrix.fdn.fr", "@belette:uc.neviani.fr", "@afriqs:matrix.fdn.fr", "@asmadeus:codewreck.org", "@tom28:matrix.fdn.fr"];

    println!("[Database]");

    // _connexion à la db ou création de la db si n'existe pas
    let connection_db =
        match Connection::open("worterkasten.db") {
            Ok(connection_db_ctrl) => {
                println!(" > Database opened");
                connection_db_ctrl
            }
            Err(e) => {
                println!("!!! Error opening database: {}", e);
                return
            }
         };

    // _initialisation de la db
    let _init_db =
        match init_db(&connection_db, &mut trigger_word_list) {
            Ok(init_db_ctrl) => {
                println!(" > Database initialized with {} words", init_db_ctrl);
                init_db_ctrl
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
                println!(" > matrix-commander lauched: {}", matrix_pid_ctrl.pid);
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
            if trigger.contains("botbotv2") && reply_check !=  '>' {

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
                let clean_message        = String::from(raw_data[3]);
                let incoming_message = Message{_room_origin: clean_room, room_id: clean_room_id, sender_id: clean_sender_id, sender_name: clean_sender_name, m_message: clean_message};
                let _answer =
                    match incoming_message.thinking(&mut trigger_word_list, &connection_db){
                        Ok(answer_ctrl) => {
                            println!("botbot: {}", answer_ctrl);
                            let _talking_status =
                                match incoming_message.talking(answer_ctrl){
                                    Ok(child) => child.id(),
                                    Err(e) => {
                                        println!("ERROR talking: {}", e);
                                        0
                                    },
                                };
                        }
                        Err(e) => {
                            println!("ERROR: {}", e);
                            line_from_buffer.clear();
                            continue
                        }
                    };
            }
        }
        // _vide la zone de lecture du buffer à chaque boucle
        line_from_buffer.clear();
    }
}
