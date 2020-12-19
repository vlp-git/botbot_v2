use std::io;

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
        println!("{}", self.m_message);
        println!("{}", self.room_origin);
        println!("{}", self.sender_id);
        println!("{}", self.sender_name);
        println!("{}", self.m_message);
    }
}

fn clean_room_origin(raw_room_origin:String) -> String {
    return String::from("plop")
}

fn clean_room_id(raw_room_id:String) -> String {
    return String::from("plop")
}

fn main() {
    loop {

        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer){
            Ok(_) => {
                // check que le message correspond bien à une entrée correcte de matrix-commander: https://github.com/8go/matrix-commander
                let raw_data: Vec<&str> = buffer.split('|').collect();
                if raw_data.len() == 4 {
                    // check du mot clef botbot peu importe le case
                    let mut trigger = String::from(raw_data[3]);
                    trigger.make_ascii_lowercase();
                    if trigger.contains("botbot") {
                        // construction du Message
                        let incoming_message = Message{room_origin: clean_room_origin(String::from(raw_data[0])), room_id: clean_room_origin(String::from(raw_data[0])), sender_id: String::from("empty"), sender_name: String::from("empty"), m_message: String::from(raw_data[3])};
                        incoming_message.print_full_message();
                    }
                }
            }
            Err(e) => {
                println!("ERROR: {}", e)
            }
        }
    }
}
