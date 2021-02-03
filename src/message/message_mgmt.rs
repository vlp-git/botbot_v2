////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION diverses de gestions des messages

// _renvoie la liste adminsys ou admincore pour _ping
pub fn get_admin_list(sender: &String, my_list: &Vec<String>) ->  Result<String, String> {
    let mut iterator_sys = my_list.iter();
    let mut raw_liste_to_ping = String::from("ping: ");
    while let Some(x) = iterator_sys.next() {
        let fin_mark =
            match x.find(":") {
                Some(fin_mark_index) => fin_mark_index,
                None => continue,
            };
        let admin_name_to_add = &x[1..fin_mark];
        if admin_name_to_add !=  sender {
            raw_liste_to_ping += admin_name_to_add;
            raw_liste_to_ping += ", ";
        }else{
            continue
        }
    }
    let liste_to_ping = &raw_liste_to_ping[0..raw_liste_to_ping.len()-2];
    Ok(liste_to_ping.to_string())
}
