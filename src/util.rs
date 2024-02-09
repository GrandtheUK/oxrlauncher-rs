use std::{
    path::PathBuf, process, sync::mpsc::Sender, thread, time::Duration};
use steamlocate::SteamDir;
use sysinfo::Pid;


pub fn get_installed_steam_games() -> Vec<Game> {
    let mut games: Vec<Game> = Vec::new();
    let mut steamdir = SteamDir::locate().unwrap();
    let mut s = steamdir.clone();
    

    let game_list = steamdir.apps();
    let libraryfolders = s.libraryfolders();
    let paths = &libraryfolders.paths;
    for path in paths {
        let p = path.clone();
        println!("found library folder: {}", p.into_os_string().to_str().unwrap());
    }

    for game in game_list {
        match game.1 {
            Some(app) => {
                let name = app.clone().name.unwrap();
                if name.contains("Steamworks") || name.contains("Proton") || name.contains("Linux Runtime") || name.contains("SteamVR") {
                    continue;
                }
                let game = Game::new_steam(app.appid, app.name.as_ref().unwrap().to_owned());
                games.push(game);
            },
            None => {},
        }
    }
    games
}

#[derive(Debug,Clone)]
pub enum Kind {
    STEAM,
    NONSTEAM,
}

#[derive(Debug,Clone)]
pub struct Game 
{
    pub name: String,
    kind: Kind,
    pub steamid: Option<u32>,
    pub path: Option<PathBuf>,
}

// steam game impl 
impl Game {
    pub fn new_steam(id: u32, name: String) -> Self {
        Self {
            name, 
            kind: Kind::STEAM, 
            steamid: Some(id), 
            path: None,
        }
    }

    fn run_steam(self, tx: Sender::<u32>) {
        let id = self.steamid.unwrap().to_string();
        let url = format!("steam://launch/{}/vr",id);
        match process::Command::new("xdg-open").arg(url.as_str()).stdout(process::Stdio::null()).output() {
            Ok(_) => (),
            Err(_) => println!("couldn't open steam app"),
        }
        // let (tx,rx) = channel::<u32>();
        println!("pre-thread");
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(10));
            println!("sleep over");
            let sys = sysinfo::System::new();
            let mut pid: Pid = Pid::from_u32(0);
            let processes = sys.processes_by_exact_name("reaper");
            for process in processes {
                println!("pid: {}",process.pid().as_u32());
                // if process.pid().as_u32() != 0 {   
                    pid = process.pid();
                    break;
                // }
            }
            if pid.as_u32() != 0 {
                let process = sys.process(pid).unwrap();
                println!("{:#?}",process.cmd().last());
                let _ = tx.send(pid.as_u32());
            }
            match process::Command::new("pgrep").arg("-lP").arg(pid.as_u32().to_string()).output() {
                Ok(out) => {
                    println!("output of pgrep: {:#?}", out.stdout);
                },
                Err(e) => {
                    println!("pgrep failed: {}",e);
                }
            }
            println!("thread end");
        });
    }
}

// non-steam game impl
impl Game {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self { 
            name: name, 
            kind: Kind::NONSTEAM, 
            steamid: None, 
            path: Some(path) 
        }
    }

    fn run_non_steam(self, tx: Sender::<u32>) {
        match process::Command::new(self.path.unwrap().as_os_str()).spawn() {
            Ok(child) => {
                // println!("opened app successfully");
                let _ = tx.send(child.id());
            },
            Err(_) => {
                println!("couldn't open app");
            },
        }
    }
}

impl Game {
    pub fn run(self, tx: Sender::<u32>) {
        match self.kind {
            Kind::STEAM => self.run_steam(tx),
            Kind::NONSTEAM => self.run_non_steam(tx),
        }
    }
}