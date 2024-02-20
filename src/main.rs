use std::{f32::consts::PI, sync::mpsc, process};
use glam::{Vec2, Vec3, Quat};
use stereokit::{Pose, SettingsBuilder, StereoKitMultiThread, ButtonState, WindowType, MoveType, Handed};
use steam_webapi_rust_sdk::{get_app_details,get_cached_app_details};

mod util;
use sysinfo::System;
use util::*;

#[derive(Clone)]
struct OxrLauncherData {
    pub visibility: bool,
    pub pose: Pose,
    dimensions: Vec2,
    pub status: LauncherState,
    games: Vec<Game>
}

impl OxrLauncherData {
    fn new() -> Self {
        OxrLauncherData {
            visibility: true,
            pose: Pose::new( Vec3::new( 0.0,-0.25,-1.0 ), Quat::default().mul_quat(Quat::from_rotation_y(PI)) ),
            dimensions: Vec2::new( 0.75, 0.5 ),
            // pid: None,
            status: LauncherState::GameNotStarted,
            games: Vec::<Game>::new()
        }
    }
}


fn main() {
    let sk = SettingsBuilder::new()
        .app_name("OpenXRLauncher")
        .disable_desktop_input_window(true)
        .display_preference(stereokit::DisplayMode::MixedReality)
        .overlay_priority(u32::MAX)
        .init()
        .unwrap();
    let mut launcher = OxrLauncherData::new();
    let games = get_installed_steam_games();
    let mut filtered_games = Vec::<Game>::new();
    let library_icon_path = String::from("/home/ben/.steam/steam/appcache/librarycache/");

    println!("Attempting to get Appdetails of installed games through SteamCMD API and storing in cache. If no games show up, then accessing local cache or API may have failed.");
    for game in games {
        let details = match get_cached_app_details(game.steamid.unwrap().into()) {
            Ok(detail) => {
                detail
            },
            Err(_) => match get_app_details(game.steamid.unwrap().into()) {
                Ok(detail) => {
                    detail
                },
                Err(_) => {
                    println!("could not get appdetails for game {}", game.name);
                    continue
                },
            },
        };
        for category in details.categories.into_iter() {
            if category.id == 53 || category.id == 54  {
                filtered_games.push(game.clone());
            }
        }
    }

    let (tx_state,rx_state) = mpsc::channel::<LauncherState>();

    launcher.games.append(&mut filtered_games);

    sk.run(|sk| {
        let mut head = sk.input_head();
        head.orientation.x = 0.0;
        head.orientation.z = 0.0;
        let _ = head.orientation.normalize();

        // get vector for controller to headset
        let palm = sk.input_controller(Handed::Left).palm;
        let to_face = sk.input_controller(Handed::Left).pose.position - sk.input_head().position;

        match rx_state.try_recv() {
            Ok(value) => launcher.status = value,
            Err(_) => (),
        }
        
        // open menu on controller menu and grip when controller palm is facing headset
        if sk.input_controller_menu().contains(ButtonState::JUST_ACTIVE) && sk.input_controller(Handed::Left).grip > 0.5 && to_face.dot(palm.forward()).abs() < 0.15  {
            println!("Menu pressed");
            launcher.visibility = !launcher.visibility;
            if launcher.visibility == true {
                let pos: Vec3 = head.position + head.orientation.mul_vec3(0.7 * Vec3::NEG_Z) + 0.125 * Vec3::Y;
                launcher.pose = Pose::new(pos, head.orientation.mul_quat(Quat::from_rotation_y(PI)))
            } 
        }

        if launcher.visibility {
            let games = launcher.games.clone();
            sk.window("title",&mut launcher.pose, launcher.dimensions, WindowType::Normal, MoveType::FaceUser, |ui| {
                match launcher.status {
                    LauncherState::GameNotStarted => {
                        for game in games {
                            let sprite_path = library_icon_path.clone()+game.steamid.unwrap().to_string().as_str()+"_library_600x900.jpg";
                            match sk.sprite_create_file(sprite_path, stereokit::SpriteType::Single, "0".to_string()) {
                                Ok(sprite) => (),
                                Err(e) => println!("Error loading sprite: {}",e),
                            }
                            // println!("{}",game.name);
                            ui.same_line();
                            let name = game.name.clone();
                            if ui.button(&name) {
                                println!("starting {}",&name);
                                game.run(tx_state.clone());
                            }
                        };
                    },
                    LauncherState::SteamGameRunning(steamid) => {
                        let name = get_cached_app_details(steamid as i64).unwrap().name;
                        ui.label(format!("Currently playing: {}",name), true);
                        if ui.button("kill game") {
                            let sys = System::new_all();
                            let mut processes = sys.processes_by_exact_name("reaper");
                            loop {
                                match processes.next() {
                                    Some(process) => {
                                        for command in process.cmd() {
                                            if command.contains("common") && !command.contains("entry-point") {
                                                let binary_path: Vec<&str> = command.split("/").collect();
                                                let binary = binary_path.last().unwrap().to_string();
                                                
                                                if binary.to_owned() == "proton" {
                                                    let proton = sys.processes_by_name("pressure-vessel").next().unwrap();
                                                    let ppid = proton.pid().as_u32();
            
                                                    let _ = process::Command::new("pkill").arg("-9").arg("python3").arg("-n").output().unwrap();
                                                    let _ = process::Command::new("pkill").arg("-9").arg("-P").arg(ppid.to_string()).arg("-n").output().unwrap();
                                                    println!("end of run_steam tracking. program should be dead");
                                                    break;
                                                } else {
                                                    let _ = process::Command::new("pkill").arg("-9").arg("-f").arg(binary).output().unwrap();
                                                }
                                            }
                                        }
                                    },
                                    None => break,
                                }
                            };
                            launcher.status = LauncherState::GameNotStarted;
                        }
                    }
                    LauncherState::GameRunning(_pid) => (), // TODO: Non-steam game
                }
            });
        }
        
    }, |_| {});
}

