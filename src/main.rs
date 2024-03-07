use std::{f32::consts::PI, sync::mpsc, process, collections::HashMap, thread, time::Duration};
use glam::{Vec2, Vec3, Quat};
use stereokit::{Pose, SettingsBuilder, StereoKitMultiThread, ButtonState, WindowType, MoveType, Handed, Sprite, DisplayMode, DisplayBlend, DepthMode};
use steam_webapi_rust_sdk::{get_app_details,get_cached_app_details};
use dirs::home_dir;

mod util;
use sysinfo::System;
use util::*;

#[derive(Clone)]
struct OxrLauncherData {
    pub visibility: bool,
    pub pose: Pose,
    dimensions: Vec2,
    pub status: LauncherState,
    games: Vec<Game>,
}

impl OxrLauncherData {
    fn new() -> Self {
        OxrLauncherData {
            visibility: false,
            pose: Pose::new( Vec3::new( 0.0,-0.25,-1.0 ), Quat::default().mul_quat(Quat::from_rotation_y(PI)) ),
            dimensions: Vec2::new( 0.75, 0.5 ),
            // pid: None,
            status: LauncherState::GameNotStarted,
            games: Vec::<Game>::new(),
        }
    }
}


fn main() {
    // let sk = SettingsBuilder::new()
    //     .app_name("OpenXRLauncher")
    //     .overlay_app(true)
    //     .overlay_priority(u32::MAX)
    //     .display_preference(DisplayMode::MixedReality)
    //     .blend_preference(DisplayBlend::AnyTransparent)
    //     .init()
    //     .unwrap();
    let sk = stereokit::Settings {
        app_name: "OpenXRLauncher".to_string(),
        display_preference: DisplayMode::MixedReality,
        blend_preference: DisplayBlend::AnyTransparent,
        depth_mode: DepthMode::D32,
        overlay_app: true,
        overlay_priority: u32::MAX,
        disable_desktop_input_window: true,
        ..Default::default()
    }
    .init()
    .expect("StereoKit init fail!");

    let mut launcher = OxrLauncherData::new();
    let games = get_installed_steam_games();
    let mut filtered_games = Vec::<Game>::new();
    let library_icon_path: String = home_dir().unwrap().as_mut_os_string().to_str().unwrap().to_string() + "/.steam/steam/appcache/librarycache/";
    let mut library_icons: HashMap<u32, Sprite> = HashMap::new();
    let mut library_banners: HashMap<u32, Sprite> = HashMap::new();

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

                let icon_path = library_icon_path.clone()+game.steamid.unwrap().to_string().as_str()+"_library_600x900.jpg";
                let icon_sprite = sk.sprite_create_file(icon_path, stereokit::SpriteType::Single, "0".to_string()).unwrap();

                let banner_path = library_icon_path.clone()+game.steamid.unwrap().to_string().as_str()+"_header.jpg";
                let banner_sprite = sk.sprite_create_file(banner_path, stereokit::SpriteType::Single, "0".to_string()).unwrap();


                library_banners.insert(game.steamid.unwrap().clone(), banner_sprite);
                library_icons.insert(game.steamid.unwrap().clone(), icon_sprite);
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
            sk.input_hand_visible(Handed::Left, true);
            sk.input_hand_visible(Handed::Right, true);
            let games = launcher.games.clone();
            sk.window("OpenXR Launcher",&mut launcher.pose, launcher.dimensions, WindowType::Normal, MoveType::FaceUser, |ui| {
                match launcher.status {
                    LauncherState::GameNotStarted => {
                        for game in games {
                            let name = game.name.clone();
                            let sprite = library_icons.get_mut(&game.steamid.unwrap()).unwrap();
                            // println!("{}",game.name);
                            ui.same_line();
                            
                            if ui.button_img_size(&"", sprite, stereokit::UiBtnLayout::Centre, Vec2::new(0.08,0.12)) {
                                println!("starting {}",&name);
                                game.run(tx_state.clone());
                            }
                        };
                    },
                    LauncherState::SteamGameRunning(steamid) => {
                        let sprite = library_banners.get_mut(&steamid).unwrap();
                        ui.image(sprite,Vec2::new(0.6,0.28));
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
                                                println!("{:#?}", command);
                                                let binary_path: Vec<&str> = command.split("/").collect();
                                                let binary = binary_path.last().unwrap().to_string();
                                                
                                                if binary.to_owned() == "proton" {
                                                    let mut proton: Option<&sysinfo::Process> = None;
                                                    for process in sys.processes_by_name("pressure-vessel") {
                                                        let search = format!("SteamAppId={}",steamid);
                                                        if process.environ().contains(&search) {
                                                            proton = Some(process);
                                                        }
                                                    }
                                                    match proton {
                                                        Some(_) => println!("got a process"),
                                                        None => break,  
                                                    };
                                                    let ppid = proton.unwrap().pid().as_u32();
                                                    println!("killing PID {} and children",ppid);
                                                    // why am i killing it twice? ask god for i dont have the answer
                                                    thread::spawn(move || {
                                                        let _ = process::Command::new("pkill").arg("-9").arg("-P").arg(ppid.to_string()).arg("python").output().unwrap();
                                                        let _ = process::Command::new("pkill").arg("-9").arg("-P").arg(ppid.to_string()).arg("wine64").output().unwrap();
                                                        let _ = process::Command::new("pkill").arg("-9").arg("-P").arg(ppid.to_string()).output().unwrap();
                                                        let _ = process::Command::new("pkill").arg("-9").arg("-P").arg(ppid.to_string()).output().unwrap();
                                                    });
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
        } else {
            sk.input_hand_visible(Handed::Left, false);
            sk.input_hand_visible(Handed::Right, false);
        }
        
    }, |_| {});
}

