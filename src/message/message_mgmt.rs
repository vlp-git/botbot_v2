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

// _récupère l'argument de gauche contenu dans entre crochets
pub fn get_left_arg(admin_msg: &String) -> Result<String, String> {
    // _trouve l'index de [ de l'argument de gauche
    let debut_mark =
        match admin_msg.find("[") {
            Some(debut_mark_index) => debut_mark_index + 1,
            None => return Err("ERROR: unable to find left arg start".to_string()),
        };
    // _trouve l'index de ] de l'argument de gauche
    let fin_mark =
        match admin_msg.find("]") {
            Some(fin_mark_index) => fin_mark_index,
            None => return Err("ERROR: unable to find left arg end".to_string()),
        };
    // _si arg est vide ERROR sinon retour de la value
    if debut_mark == fin_mark {
        Err("ERROR: no value in left arg".to_string())
    }
    else {
        Ok(admin_msg[debut_mark..fin_mark].to_string())
    }
}

// _récupère l'argument de droite contenu dans entre crochets
pub fn get_right_arg(admin_msg: &String) -> Result<String, String> {
    // _trouve l'index de [ de l'argument de droite
    let debut_mark =
        match admin_msg.rfind("[") {
            Some(debut_mark_index) => debut_mark_index + 1,
            None => return Err("ERROR: unable to find right arg start".to_string()),
        };
    // _trouve l'index de ] de l'argument de droite
    let fin_mark =
        match admin_msg.rfind("]") {
            Some(fin_mark_index) => fin_mark_index,
            None => return Err("ERROR: unable to find right arg end".to_string()),
        };
    // _si arg est vide ERROR sinon retour de la value
    if debut_mark == fin_mark {
        Err("ERROR: no value in right arg".to_string())
    }
    else {
        Ok(admin_msg[debut_mark..fin_mark].to_string())
    }
}
