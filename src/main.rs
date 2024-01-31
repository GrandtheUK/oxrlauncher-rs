use std::f32::consts::PI;
use device_query::{DeviceQuery,DeviceState,Keycode};
use glam::{Vec2, Vec3, Quat};
use stereokit::{Pose, SettingsBuilder, StereoKitMultiThread, ButtonState, WindowType, MoveType, Handed};
use steam_webapi_rust_sdk::{get_app_details,get_cached_app_details};

mod util;
use util::*;

#[derive(Clone)]
struct OxrLauncherData {
    pub visibility: bool,
    pub pose: Pose,
    dimensions: Vec2,
    pub pid: Option<u32>,
    games: Vec<Game>
}

impl OxrLauncherData {
    fn new() -> Self {
        OxrLauncherData {
            visibility: true,
            pose: Pose::new( Vec3::new( 0.0,-0.25,-1.0 ), Quat::default().mul_quat(Quat::from_rotation_y(PI)) ),
            dimensions: Vec2::new( 0.75, 0.5 ),
            pid: None,
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
        
        if launcher.visibility {
            let games = launcher.games.clone();
            sk.window("title",&mut launcher.pose, launcher.dimensions, WindowType::Normal, MoveType::FaceUser, |ui| {
                if let Some(_) = launcher.pid {
                    
                } else {
                    for game in games {
                        // println!("{}",game.name);
                        ui.same_line();
                        let name = game.name.clone();
                        ui.button(name).then(|| {
                            println!("game start");
                            launcher.pid = game.run();
                        });
                    }
                }
            });
        }
        
    }, |_| {});
}

