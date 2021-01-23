use std::vec::*;
use sqlite::{Connection, State};

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION initialisation de la db

// _initialise la db
pub fn init_db () -> (Result<Connection, String>, Vec<String>) {
    let mut trigger_word_list: Vec<String> = Vec::new();
    let connection_db =
       match Connection::open("worterkasten.db") {
           Ok(connection_db_ctrl) => connection_db_ctrl,
           Err(_e) => return (Err("Talking table fail to initialized".to_string()),trigger_word_list),
        };
    {
        // _crée la table talking si elle n'existe pas
        let mut create_table_statement =
            match connection_db.prepare("CREATE TABLE if not exists talking (chat_id INTEGER PRIMARY KEY, trigger TEXT not null, answer TEXT not null);") {
                Ok(create_table_statement_ctrl) => create_table_statement_ctrl,
                Err(_e) => return (Err("Talking table fail to initialized".to_string()), trigger_word_list),
              };

        while let State::Row = create_table_statement.next().unwrap() {};
    }
    {
        // _charge dans trigger_word_list tous les triggers de la table talking
        let mut add_words_statement =
            match connection_db.prepare("SELECT trigger FROM talking") {
                Ok(add_words_statement_ctrl) => add_words_statement_ctrl,
                Err(_e) => return (Err("Fail to load wordlist.db".to_string()), trigger_word_list),
              };

        while let State::Row = add_words_statement.next().unwrap() {
                let word_to_add = add_words_statement.read::<String>(0).unwrap();
                if !trigger_word_list.contains(&word_to_add){
                    trigger_word_list.push(word_to_add);
                }
            }
    }
    (Ok(connection_db), trigger_word_list)
}
