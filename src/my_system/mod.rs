use std::process::Command;
use regex::Regex;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION de lancement du processus matrix_commander
///bin/df -hk /var | tail -1 | tr -s ' ' | cut -d' ' -f5

// _initialise le daemon matrix-commander
pub fn monit_disk_space() -> String {

    let mut disk_status: Vec<String> = Vec::new();

    let space_left_command = Command::new("df")
        .output()
        .expect("failed to execute process");

    let space_left = String::from_utf8_lossy(&space_left_command.stdout);

    let disk_list: Vec<&str> = space_left.split('\n').collect();

    let disk_to_search_re = "/dev/vdb".to_string();
    let disk_re =
        match Regex::new(&disk_to_search_re){
            Ok(disk_re_ctrl) => disk_re_ctrl,
            Err(_e) => {
                return "ERROR: fail to build system regex".to_string();
            }
        };

    for line in disk_list {
        if disk_re.is_match(&line){
            let data_list: Vec<&str> = line.split(' ').collect();
            for data in data_list{
                if data != ""{
                    disk_status.push(data.to_string());
                }
            }
        }
    }

    return format!("Disk usage: {}", disk_status[4]);
}
