use std::path::PathBuf;
use std::{process, thread, time};
use std::{process::{Command, Stdio}, io::{BufReader, BufRead}};
use regex::Regex;
use home::home_dir;
use sysinfo::System;

pub fn get_installed_steam_games() -> Option<Vec<String>> {
    let steam: String = home_dir().unwrap().to_str().unwrap().to_string() + "/.steam/root";
    let pt = Command::new("protontricks")
        .arg("-l")
        .stdout(Stdio::piped())
        .env("STEAM_DIR", &steam)
        .spawn()
        .expect("protontricks not found");

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

#[derive(Clone)]
pub enum Kind {
    STEAM,
    NONSTEAM,
}

#[derive(Clone)]
pub struct Game 
{
    // pub name: String,
    kind: Kind,
    steamid: Option<u32>,
    path: Option<PathBuf>,
}

// steam game impl 
impl Game {
    pub fn new_steam(id: u32, name: String) -> Self {
        Self {
            // name, 
            kind: Kind::STEAM, 
            steamid: Some(id), 
            path: None,
        }
    }

    async fn get_pid(self) -> Option<u32> {
        let mut pids: Vec<u32> = Vec::<u32>::new();
        if let Kind::NONSTEAM = self.kind {
            return None
        } else {
            let sys = System::new_all();
            let ten_seconds = time::Duration::from_millis(10000);
            thread::sleep(ten_seconds);
            for process in sys.processes_by_name("Beat Saber") {
                let pid = process.pid();
                println!("Beat saber has pid: {}",pid.as_u32());
                pids.push(pid.as_u32());
            }
            Some(pids[0])
        }
    }

    async fn run_steam(self) -> u32 {
        let id = self.steamid.unwrap().to_string();
        let url = String::from("steam://rungameid/")+id.as_str();
        match process::Command::new("xdg-open").arg(url.as_str()).output() {
            Ok(_) => println!("opened steam app successfully"),
            Err(_) => println!("couldn't open steam app"),
        }
        self.get_pid().await.unwrap()
    }
}

// non-steam game impl
impl Game {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self { 
            // name: name, 
            kind: Kind::NONSTEAM, 
            steamid: None, 
            path: Some(path) 
        }
    }

    async fn run_non_steam(self) -> Option<u32> {
        match process::Command::new(self.path.unwrap().as_os_str()).spawn() {
            Ok(c) => {
                println!("opened app successfully");
                Some(c.id())
            },
            Err(_) => {
                println!("couldn't open app");
                None
            },
        }
    }
}

impl Game {
    pub async fn run(self) -> u32 {
        match self.kind {
            Kind::STEAM => self.run_steam().await,
            Kind::NONSTEAM => self.run_non_steam().await.unwrap(),
        }
    }
}