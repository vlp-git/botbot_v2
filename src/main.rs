use std::io;
use std::process::Command;

struct Message{
    // structure d'un message reçu
    room_origin: String,
    room_id: String,
    sender_id: String,
    sender_name: String,
    m_message: String,
}

impl Message{
    fn print_full_message(&self){
        println!("{}", self.room_origin);
        println!("{}", self.room_id);
        println!("{}", self.sender_id);
        println!("{}", self.sender_name);
        println!("{}", self.m_message);
    }
    fn thinking(&self){
        println!("Hello {} You just say: {}", self.sender_name, self.m_message);
    }
}

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

fn main() {
    loop {
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer){
            Ok(_) => {
                // check que le message correspond bien à une entrée correcte de matrix-commander: https://github.com/8go/matrix-commander
                let raw_data: Vec<&str> = buffer.split('|').collect();
                //println!("{}", buffer);
                if raw_data.len() == 4 {
                    // check du mot clef botbot peu importe le case
                    let mut trigger = String::from(raw_data[3]);
                    trigger.make_ascii_lowercase();
                    if trigger.contains("botbot") {
                        // construction du Message
                        let incoming_message = Message{room_origin: clean_room_origin(String::from(raw_data[0])), room_id: clean_room_id(String::from(raw_data[0])), sender_id: clean_sender_id(String::from(raw_data[1])), sender_name: clean_sender_name(String::from(raw_data[1])), m_message: String::from(raw_data[3])};
                        //incoming_message.print_full_message();
                        incoming_message.thinking();
                    }
                }
            }
            Err(e) => {
                println!("ERROR: {}", e)
            }
        }
    }
}
