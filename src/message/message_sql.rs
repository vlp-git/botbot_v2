use sqlite::{Connection, State};
use rand::Rng;
use regex::Regex;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION d'échange avec la db

// _ajoute un trigger/answer dans la base
pub fn add_chat(trigger: String, answer: String, connection_db: &Connection, trigger_word_list: &mut Vec<String>) -> Result<String, String> {
    let mut insert_statement =
        match connection_db.prepare("INSERT INTO talking (trigger, answer) VALUES (?, ?);"){
            Ok(insert_statement_ctrl) => insert_statement_ctrl,
            Err(e) => return Err(format!("ERROR: add prepare db - {}", e)),
          };
        let _bind1_statement =
            match insert_statement.bind(1, &trigger[..]){
                Ok(_bind1_statement_ctrl) => _bind1_statement_ctrl,
                Err(e) => return Err(format!("ERROR: add binding trigger - {}", e)),
            };
        let _bind2_statement =
            match insert_statement.bind(2, &answer[..]){
                Ok(_bind2_statement_ctrl) => _bind2_statement_ctrl,
                Err(e) => return Err(format!("ERROR: add binding answer - {}", e)),
            };
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

// _supprime un trigger/answer dans la base
pub fn del_chat(trigger: String, connection_db: &Connection, trigger_word_list: &mut Vec<String>) -> Result<String, String> {
    if !trigger_word_list.contains(&trigger) {
        return Err(format!("ERROR: trigger not in db"))
    }
    let mut del_statement =
        match connection_db.prepare("DELETE FROM talking WHERE trigger=?"){
            Ok(del_statement_ctrl) => del_statement_ctrl,
            Err(e) => return Err(format!("ERROR: del prepare db - {}", e)),
          };
    let _bind_statement =
      match del_statement.bind(1, &trigger[..]){
          Ok(_bind_statement_ctrl) => _bind_statement_ctrl,
          Err(e) => return Err(format!("ERROR: del binding trigger- {}", e)),
      };
    let _run_statement =
        match del_statement.next() {
            Ok(_run_statement_ctrl) => _run_statement_ctrl,
            Err(e) => return Err(format!("ERROR: process del trigger - {}", e)),
        };
    trigger_word_list.retain(|x| *x != trigger);
    Ok(trigger)
}

// _récupère une answer dans la base à partir de son trigger
pub fn get_answer(choice: String, connection_db: &Connection, trigger_word_list: &mut Vec<String>) -> Result<String, String> {
    let mut tmp_answers: Vec<String> = Vec::new();
    for x in trigger_word_list {
        let re_to_search = format!("\\s{}[\\s\\?!,]", x);
        let re =
            match Regex::new(&re_to_search){
                Ok(re_ctrl) => re_ctrl,
                Err(e) => return Err(format!("ERROR: setup regex - {}", e)),
            };

        if  re.is_match(&choice) {
            let mut select_statement =
                match connection_db.prepare("SELECT answer FROM talking where trigger=?"){
                    Ok(select_statement_ctrl) => select_statement_ctrl,
                    Err(e) =>  return Err(format!("ERROR: select prepare db - {}", e)),
                  };
            let _bind_statement =
                match select_statement.bind(1, &x[..]){
                    Ok(_bind_statement_ctrl) => _bind_statement_ctrl,
                    Err(e) => return Err(format!("ERROR: select binding trigger- {}", e)),
                };
            while let Ok(State::Row) = select_statement.next() {
                let blabla =
                    match select_statement.read::<String>(0){
                        Ok(blabla_ctrl) => blabla_ctrl,
                        Err(_e) => continue,
                    };
                tmp_answers.push(blabla);
            }
            continue;
        }
    }
    if tmp_answers.len() != 0 {
        let mut rng = rand::thread_rng();
        Ok(tmp_answers[rng.gen_range(0..tmp_answers.len())].to_string())
    }else{
        Err(format!("ERROR: no word found for : {} ", choice))
    }
}
