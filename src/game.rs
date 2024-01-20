use std::process;
use std::{process::{Command, Stdio}, io::{BufReader, BufRead}};
use regex::Regex;

pub fn get_installed_steam_games() -> Option<Vec<String>> {
    let steam: String = home_dir().unwrap().to_str().unwrap().to_string() + "/.steam/root";
    let pt = Command::new("protontricks")
        .arg("-l")
        .stdout(Stdio::piped())
        .env("STEAM_DIR", &steam)
        .spawn()
        .unwrap();

    let mut res: Vec<String> = Vec::new();
    let reg = Regex::new(r"\([0-9]+[0-9]?\)").unwrap();
    let reg2 = Regex::new(r"[0-9]+[0-9]?").unwrap();
    match pt.stdout {
        Some(out) => {
            let buf = BufReader::new(out);
            for line in buf.lines() {
                match line {
                    Ok(l) => {
                        for cap in reg.captures_iter(l.as_str()) {
                            let mut id = cap[0].to_string();
                            id = id[1..id.len()-1].to_string();
                            res.push(id);
                        }
                    },
                    Err(_) => {
                        println!("line is error");
                        return None
                    }
                }
            }
        }
        None => {
            println!("stdout is none");
            return None
        }
    }
    Some(res)

}

pub fn start_steam(appid:u32) {
    let id = appid.to_string();
    let url = String::from("steam://rungameid/")+id.as_str();
    match process::Command::new("xdg-open").arg(url.as_str()).output() {
        Ok(_) => println!("opened steam app {} successfully", appid),
        Err(_) => println!("couldn't open steam app {}", appid),
    }
}