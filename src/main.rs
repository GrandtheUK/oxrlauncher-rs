use std::{
    process,
    thread,
    f32::consts::PI
};
mod game;

use glam::*;
use stereokit::*;
use sysinfo::System;
use game::get_installed_steam_games;

#[derive(Clone,Copy)]
struct OxrLauncherData {
    pub visibility: bool,
    pose: Pose,
    dimensions: Vec2,
    pub pid: Option<u32>,

}

impl OxrLauncherData {
    fn new() -> Self {
        OxrLauncherData {
            visibility: true,
            pose: Pose::new( vec3( 0.0,-0.25,-1.0 ), Quat::default().mul_quat(Quat::from_rotation_y(PI)) ),
            dimensions: vec2( 1.5, 1.0 ),
            pid: None,
        }
    }
    // get/set visibility of the launcher
    pub fn visible(self) -> bool {
        self.visibility
    }
    pub fn flip_vis(mut self) {
        self.visibility = !self.visibility;
    }

    // draw the menu in preparation for transferring to the plane
    pub fn construct_menu(mut self, sk: &SkDraw) {
        sk.window("window_title", self.pose, self.dimensions, WindowType::Normal, MoveType::FaceUser, |ui| {
            ui.button("Beat Saber").then(|| {
                self.start_steam(620980);
                let sys = System::new_all();
                // let ten_seconds = time::Duration::from_millis(10000);
                // thread::sleep(ten_seconds);
                for process in sys.processes_by_name("Beat Saber") {
                    let pid = process.pid();
                    println!("Beat saber has pid: {}",pid.as_u32());
                    self.pid = Some(pid.as_u32());
                }
            });
        });
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
        if sk.input_controller_menu().contains(ButtonState::JUST_ACTIVE) {
            println!("Menu pressed");
            if launcher.visible() {
            // if menu_visible == false {
                // set position of menu to be drawn
                let pos: Vec3 = head.position + head.orientation.mul_vec3(0.5 * Vec3::NEG_Z) + 0.25 * Vec3::Y;
                launcher.pose = Pose::new(pos, head.orientation.mul_quat(Quat::from_rotation_y(PI)))
            } 
            // menu_visible = !menu_visible;
            launcher.flip_vis();
            
        }

        // if sk.input_controller_menu().contains(ButtonState::ACTIVE) && !(sk.input_controller(Handed::Left).grip == 1.0) {
        //     start_steamvr(620980);
        // }
        
        if launcher.visible() {
        // if menu_visible {
            // draw menu
            launcher.construct_menu(sk);
            

            // sk.mesh_draw(plane, Material::UI, launcher.pos, BLUE, RenderLayer::LAYER0);
        }
        
    }, |_| {});
}

