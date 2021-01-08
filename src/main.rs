////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  botbot v2 by vlp

use std::io::{BufRead, BufReader, Error};
use std::process::{Command, Stdio, Child};
use sqlite::{Connection, State};
use unidecode::unidecode;
use procfs::process::Process;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  Structure et traits des messages reçus

struct Message{
    // structure d'un message reçu
    _room_origin: String,
    room_id: String,
    sender_id: String,
    sender_name: String,
    m_message: String,
    m_answer: String,
}

impl Message{
    fn _print_full_message(&self){
        println!("{}", self._room_origin);
        println!("{}", self.room_id);
        println!("{}", self.sender_id);
        println!("{}", self.sender_name);
        println!("{}", self.m_message);
        println!("{}", self.m_answer);
    }
    fn thinking(&self, trigger_word_list: &mut Vec<String>, connection_db: &Connection) -> String {
        let mut choice = String::from(unidecode(&self.m_message).to_string());
        ///// to split uppercases
        choice.make_ascii_lowercase();
        if choice.contains("admin add") && &self.sender_id == "@vlp:matrix.fdn.fr" {
            let check_add = add_chat(get_left_arg(&choice),get_right_arg(&choice), connection_db, trigger_word_list);
            if check_add != "ERROR" {
                let admin_add_return = format!("[admin mode by: {}] {} ajouté !", &self.sender_name, check_add);
                return  admin_add_return;
            } else {
                return check_add;
            }
        } else if choice.contains("admin del") && &self.sender_id == "@vlp:matrix.fdn.fr"{
            let check_del = del_chat(get_left_arg(&choice), connection_db, trigger_word_list);
            if check_del != "ERROR" {
                let admin_del_return = format!("[admin mode by: {}] {} supprimé !", &self.sender_name, check_del);
                return admin_del_return;
            } else {
                return check_del;
            }
        } else{
            let answer = return_answer(choice, connection_db, trigger_word_list);
            let modified_t_answer= &answer[..].replace("%s", &self.sender_name);
            let modified_t_answe2r= &modified_t_answer[..].replace("%n", "\n");
            return modified_t_answe2r.to_string();
        }
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

fn add_chat(trigger: String, answer: String, connection_db: &Connection, trigger_word_list: &mut Vec<String>) -> String{
    let mut insert_statement =
        match connection_db.prepare("INSERT INTO talking (trigger, answer) VALUES (?, ?);"){
            Ok(add_chat) => {
                add_chat
            }
            Err(e) => {
                println!("Error add word: {}", e);
                return "ERROR".to_string();
                }
          };
        insert_statement.bind(1, &trigger[..]).unwrap();
        insert_statement.bind(2, &answer[..]).unwrap();
        let _run_statement =
            match insert_statement.next() {
                Ok(run_ok) => run_ok,
                Err(_e) => return "ERROR".to_string(),
            };
        trigger_word_list.push(trigger.to_string());
        println!(" > admin: add '{}'", trigger);
        return trigger;
}

fn del_chat(trigger: String, connection_db: &Connection, trigger_word_list: &mut Vec<String>) -> String{
    let mut del_statement =
        match connection_db.prepare("DELETE FROM talking WHERE trigger=?"){
            Ok(check_chat) => {
                check_chat
            }
            Err(e) => {
                println!("Error del word: {}", e);
                return "ERROR".to_string();
                }
          };
    del_statement.bind(1, &trigger[..]).unwrap();
    let _run_statement =
        match del_statement.next() {
            Ok(run_ok) => run_ok,
            Err(_e) => return "ERROR".to_string(),
        };
    trigger_word_list.retain(|x| *x != trigger);
    println!(" > admin: del '{}'", trigger);
    return trigger;
}

fn return_answer(choice: String, connection_db: &Connection, trigger_word_list: &mut Vec<String>) -> String {
    for x in trigger_word_list {
        if choice.contains(&x[..]) {
            let mut select_statement =
                match connection_db.prepare("SELECT answer FROM talking where trigger=?"){
                    Ok(match_word) => {
                        match_word
                    }
                    Err(e) => {
                        println!("Error select word: {}", e);
                        return "ERROR".to_string();
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
    return "ERROR".to_string()
}
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTIONS pour nettoyer les trames de matrix-commander

fn clean_room_origin(raw_room_origin:String) -> Result<String, String> {
    let debut = match raw_room_origin.find("room") {
        Some(debut_index) => {
             debut_index + 5
         }
         None => {
             0
         }
    };
    let fin = match raw_room_origin.find("[") {
        Some(fin_index) => {
             fin_index - 1
         }
         None => {
             0
         }
    };
    if debut == 0 || fin == 0 || debut > fin {
        return Err("clean_room ERROR: Matrix-Commander output unreadable".to_string());
    }
    let clean_room_origin = &raw_room_origin[debut..fin];
    return Ok(clean_room_origin.to_string())
}

fn clean_room_id(raw_room_id:String) -> Result<String, String> {
    let debut = match raw_room_id.find("[") {
        Some(debut_index) => {
             debut_index + 1
         }
         None => {
             0
         }
    };
    let fin = match raw_room_id.find("]") {
        Some(fin_index) => {
             fin_index
         }
         None => {
             0
         }
    };
    if debut == 0 || fin == 0 || debut > fin {
        return Err("clean_room_id ERROR: Matrix-Commander output unreadable".to_string());
    }
    let clean_room_id = &raw_room_id[debut..fin];
    return Ok(clean_room_id.to_string())
}


fn clean_sender_id(raw_sender_id:String) -> Result<String, String> {
    let debut = match raw_sender_id.find("[") {
        Some(debut_index) => {
             debut_index + 1
         }
         None => {
             0
         }
    };
    let fin = match raw_sender_id.find("]") {
        Some(fin_index) => {
             fin_index
         }
         None => {
             0
         }
    };
    if debut == 0 || fin == 0 || debut > fin {
        return Err("clean_sender_id ERROR: Matrix-Commander output unreadable".to_string());
    }
    let clean_sender_id = &raw_sender_id[debut..fin];
    return Ok(clean_sender_id.to_string())
}


fn clean_sender_name(raw_sender_name:String) -> Result<String, String> {
    let debut = match raw_sender_name.find("sender") {
        Some(debut_index) => {
             debut_index + 7
         }
         None => {
             0
         }
    };
    let fin = match raw_sender_name.find("[") {
        Some(fin_index) => {
             fin_index - 1
         }
         None => {
             0
         }
    };
    if debut == 0 || fin == 0 || debut > fin {
        return Err("clean_sender_name ERROR: Matrix-Commander output unreadable".to_string());
    }
    let raw_sender_name = &raw_sender_name[debut..fin];
    return Ok(raw_sender_name.to_string())
}

fn get_left_arg(admin_msg: &String) -> String {
    let debut_mark = admin_msg.find("[").unwrap() + 1;
    let fin_mark = admin_msg.find("]").unwrap();
    return admin_msg[debut_mark..fin_mark].to_string();
}

fn get_right_arg(admin_msg: &String) -> String {
    let debut_mark = admin_msg.rfind("[").unwrap() + 1;
    let fin_mark = admin_msg.rfind("]").unwrap();
    return admin_msg[debut_mark..fin_mark].to_string();
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION initialisation de la db

fn init_db(connection_db: &Connection, trigger_word_list: &mut Vec<String>) -> Result<usize, String> {

    let mut create_table_statement =
        match connection_db.prepare("CREATE TABLE if not exists talking (chat_id INTEGER PRIMARY KEY, trigger TEXT not null UNIQUE, answer TEXT not null);") {
            Ok(create_table_statement_ctrl) => {
                create_table_statement_ctrl
            }
            Err(_e) => {
                return Err("Talking table fail to initialized".to_string());
                }
          };

    while let State::Row = create_table_statement.next().unwrap() {}

    let mut add_words_statement =
        match connection_db.prepare("SELECT trigger FROM talking") {
            Ok(add_words_statement_ctrl) => {
                add_words_statement_ctrl
            }
            Err(_e) => {
                return Err("Fail to load wordlist.db".to_string());
                }
          };

    while let State::Row = add_words_statement.next().unwrap() {
            let word_to_add = add_words_statement.read::<String>(0).unwrap();
            trigger_word_list.push(word_to_add);
        }

    return Ok(trigger_word_list.len());
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION lancement du processus matrix_commander

fn matrix_commander_daemon_launch() -> Result<Child, Error> {
    let daemon = Command::new("/home/vlp/git/matrix-commander/matrix-commander.py")
        .arg("-c/home/vlp/git/matrix-commander/credentials.json")
        .arg("-s/home/vlp/git/matrix-commander/store/")
        .arg("-lforever")
        .stdout(Stdio::piped())
        .spawn();
    return daemon;
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
            Ok(matrix_commander_ctrl) => {
                matrix_commander_ctrl
            }
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
            None => return,
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
                    break;
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
                        Err(_e) => break,
                    };
                let clean_room_id           =
                    match clean_room_id(String::from(raw_data[0])) {
                        Ok(clean_room_id_ok) => clean_room_id_ok,
                        Err(_e) => break,
                    };
                let clean_sender_id           =
                    match clean_sender_id(String::from(raw_data[1])) {
                        Ok(clean_sender_id_ok) => clean_sender_id_ok,
                        Err(_e) => break,
                    };
                let clean_sender_name           =
                    match clean_sender_name(String::from(raw_data[1])) {
                        Ok(clean_sender_name_ok) => clean_sender_name_ok,
                        Err(_e) => break,
                    };
                let clean_message        = String::from(raw_data[3]);
                let clean_answer         = String::new();
                let mut incoming_message = Message{_room_origin: clean_room, room_id: clean_room_id, sender_id: clean_sender_id, sender_name: clean_sender_name, m_message: clean_message, m_answer: clean_answer};
                incoming_message.m_answer = incoming_message.thinking(&mut trigger_word_list, &connection_db);
                if incoming_message.m_answer != "ERROR".to_string() {
                    incoming_message.talking();
                }
            }
        }
        // _vide la zone de lecture du buffer à chaque boucle
        line_from_buffer.clear();
    }

}
