use std::{path::PathBuf, process};
use steamlocate::SteamDir;


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

    fn run_steam(self) {
        let id = self.steamid.unwrap().to_string();
        let url = format!("steam://launch/{}/vr",id);
        match process::Command::new("xdg-open").arg(url.as_str()).stdout(process::Stdio::null()).output() {
            Ok(_) => (),
            Err(_) => println!("couldn't open steam app"),
        }
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

    fn run_non_steam(self) {
        match process::Command::new(self.path.unwrap().as_os_str()).spawn() {
            Ok(_) => {
                println!("opened non-steam app successfully");
            },
            Err(_) => {
                println!("couldn't open app");
            },
        }
    }
}

impl Game {
    pub fn run(self) {
        match self.kind {
            Kind::STEAM => self.run_steam(),
            Kind::NONSTEAM => self.run_non_steam(),
        }
    }
}