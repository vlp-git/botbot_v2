use std::process::Command;
use regex::Regex;

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////  FONCTION de monitoring

// _retourn l'espace utilisé du disk passé en argument
pub fn monit_disk_space(disk: String) -> Result<i32, String> {

    let mut disk_status: Vec<&str> = Vec::new();

    let space_left_command = Command::new("df")
        .output()
        .expect("failed to execute process");

    let space_left = String::from_utf8_lossy(&space_left_command.stdout);

    let disk_list: Vec<&str> = space_left.split('\n').collect();

    let disk_re =
        match Regex::new(&disk){
            Ok(disk_re_ctrl) => disk_re_ctrl,
            Err(_e) => {
                return Err("ERROR: fail to build system regex".to_string());
            }
        };


    for line in disk_list {
        if disk_re.is_match(&line){
            let data_list: Vec<&str> = line.split(' ').collect();
            for data in data_list{
                if data != ""{
                    disk_status.push(data);
                }
            }
        }
    }
    let raw_usage = disk_status[4];
    let clean_usage = &raw_usage[..raw_usage.len()-1];
    let usage =
        match clean_usage.parse::<i32>(){
            Ok(usage_ctrl) => Ok(usage_ctrl),
            Err(_e) => Err("ERROR: convert usage in Integer".to_string()),
        };

    usage
}
