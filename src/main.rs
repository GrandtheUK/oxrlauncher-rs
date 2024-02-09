use std::{f32::consts::PI, sync::mpsc, thread};
use device_query::{DeviceQuery,DeviceState,Keycode};
use glam::{Vec2, Vec3, Quat};
use stereokit::{Pose, SettingsBuilder, StereoKitMultiThread, ButtonState, WindowType, MoveType, Handed};
use steam_webapi_rust_sdk::{get_app_details,get_cached_app_details};

mod util;
use sysinfo::{Pid, ProcessStatus, System};
use util::*;

#[derive(Clone,Copy)]
enum LauncherState {
    GameNotStarted,
    GameStarting,
    GameRunning(u32),
}

#[derive(Clone)]
struct OxrLauncherData {
    pub visibility: bool,
    pub pose: Pose,
    dimensions: Vec2,
    // pub pid: Option<u32>,
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
        .overlay_app(true)
        .overlay_priority(u32::MAX)
        .init()
        .unwrap();
    let mut launcher = OxrLauncherData::new();
    let games = get_installed_steam_games();
    let mut filtered_games = Vec::<Game>::new();

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

    launcher.games.append(&mut filtered_games);

    let keyboard = DeviceState::new();

    let (pid_tx, pid_rx) = mpsc::channel::<u32>();
    let (status_tx, status_rx) = mpsc::channel::<LauncherState>();

    sk.run(|sk| {
        let mut head = sk.input_head();
        head.orientation.x = 0.0;
        head.orientation.z = 0.0;
        let _ = head.orientation.normalize();

        // get vector for controller to headset
        let palm = sk.input_controller(Handed::Left).palm;
        let to_face = sk.input_controller(Handed::Left).pose.position - sk.input_head().position;
        
        // open menu on controller menu and grip when controller palm is facing headset
        if ( sk.input_controller_menu().contains(ButtonState::JUST_ACTIVE) && sk.input_controller(Handed::Left).grip > 0.5 && to_face.dot(palm.forward()).abs() < 0.15) || keyboard.get_keys().contains(&Keycode::Escape)  {
            println!("Menu pressed");
            launcher.visibility = !launcher.visibility;
            if launcher.visibility == true {
                let pos: Vec3 = head.position + head.orientation.mul_vec3(0.7 * Vec3::NEG_Z) + 0.125 * Vec3::Y;
                launcher.pose = Pose::new(pos, head.orientation.mul_quat(Quat::from_rotation_y(PI)))
            } 
        }
        
        // Receive pid from process and track until dead
        match pid_rx.try_recv() {
            Ok(id) => {
                // launcher.pid = Some(id);
                launcher.status = LauncherState::GameRunning(id);
                let tx = status_tx.clone();
                thread::spawn(move || {
                    let sys = System::new_all();
                    let pid = Pid::from_u32(id);
                    let process = sys.process(pid).unwrap();
                    while process.status() != ProcessStatus::Dead {
                        println!("app is still alive. status is {}", process.status());
                    }
                    // launcher.pid = None;
                    let _ = tx.clone().send(LauncherState::GameNotStarted);
                });
            },
            Err(_) => ()
        }

        match status_rx.try_recv() {
            Ok(status) => {
                launcher.status = status;
            },
            Err(_) => (),
        }

        if launcher.visibility {
            let games = launcher.games.clone();
            sk.window("title",&mut launcher.pose, launcher.dimensions, WindowType::Normal, MoveType::FaceUser, |ui| {
                match launcher.status {
                    LauncherState::GameNotStarted => {
                        for game in games {
                            // println!("{}",game.name);
                            ui.same_line();
                            let name = game.name.clone();
                            if ui.button(name) {
                                launcher.status = LauncherState::GameStarting;
                                println!("game start");
                                game.run(pid_tx.clone());
                            }
                        }
                    },
                    LauncherState::GameStarting => {
                        ui.label("Starting game", true);
                    },
                    LauncherState::GameRunning(pid) => {
                        ui.label("Game running", true);
                        if ui.button("Kill") {
                            let sys = System::new_all();
                            let process = sys.process(Pid::from_u32(pid)).unwrap();
                            process.kill();
                        }
                    }
                }
            });
        }
        
    }, |_| {});
}

