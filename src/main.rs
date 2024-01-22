use std::{
    process,
    thread,
    f32::consts::PI
};
mod util;

use glam::*;
use stereokit::{sys::{ui_window_begin, ui_window_end}, *};
use sysinfo::System;
use device_query::{DeviceQuery,DeviceState,Keycode};
use util::{Game,get_installed_steam_games};

#[derive(Clone)]
struct OxrLauncherData {
    pub visibility: bool,
    pose: Pose,
    dimensions: Vec2,
    pub pid: Option<u32>,
    games: Vec<Game>
}

impl OxrLauncherData {
    fn new() -> Self {
        OxrLauncherData {
            visibility: true,
            pose: Pose::new( vec3( 0.0,-0.25,-1.0 ), Quat::default().mul_quat(Quat::from_rotation_y(PI)) ),
            dimensions: vec2( 1.5, 1.0 ),
            pid: None,
            games: Vec::<Game>::new(),
        }
    }
    // get/set visibility of the launcher
    pub fn visible(self) -> bool {
        self.visibility
    }
    pub fn flip_vis(mut self) {
        self.visibility = !self.visibility;
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
    // let mut menu_active:s bool = false;
    let mut launcher = OxrLauncherData::new();
    thread::spawn(|| {
        let games = get_installed_steam_games();
        match games {
            Some(g) => println!("things good"),
            None => println!("not good")
        }
    });
    

    let up = vec3(0.0,1.0,0.0);
    let dimensions = vec2(1.5, 1.0);

    // let mut menu_pos: Mat4 = Mat4::from_translation(vec3(0.0,0.0,-1.0));
    // let mut menu_orient: Vec3 = vec3(0.0,0.0,1.0);
    // let mut menu_visible = false;

    let keyboard = DeviceState::new();

    sk.run(|sk| {
        // menu_pos = menu_pos;
        // menu_orient = menu_orient;
        let mut head = sk.input_head();
        // let head_pos = head.position;
        //get the direction the user faces on the XZ plane (rotation around y)
        // let mut head_orient = head.orientation;
        head.orientation.x = 0.0;
        head.orientation.z = 0.0;
        let _ = head.orientation.normalize();

        
        // open menu on controller menu (not system)
        if sk.input_controller_menu().contains(ButtonState::JUST_ACTIVE) || keyboard.get_keys().contains(&Keycode::Escape)  {
            println!("Menu pressed");
            if launcher.visibility == true {
                let pos: Vec3 = head.position + head.orientation.mul_vec3(0.5 * Vec3::NEG_Z) + 0.25 * Vec3::Y;
                launcher.pose = Pose::new(pos, head.orientation.mul_quat(Quat::from_rotation_y(PI)))
            } 

            
        }
        
        if launcher.visibility {
            let games = launcher.games.clone();
            sk.window("title",launcher.pose, launcher.dimensions, WindowType::Normal, MoveType::FaceUser, |ui| {
                for game in games {
                    ui.button("a").then(|| {
                        let pid = thread::spawn(||{game.run()});
                    });
                } 
            });
        }
        
    }, |_| {});
}

