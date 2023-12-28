use std::path::PathBuf;
use glam::*;
use stereokit::{
    *,
    named_colors::*
};
use open;

#[derive(Clone,Copy)]
struct OxrLauncherData {
    menu_active: bool,
    pub pos: Mat4,
    pub orient: Vec3,
}


impl OxrLauncherData {
    fn new() -> Self {
        OxrLauncherData {
            menu_active: false,
            pos: Mat4::from_translation(vec3(0.0,0.0,-1.0)),
            orient: vec3(0.0,0.0,1.0),
        }
    }
    // get/set visibility of the launcher
    pub fn active(self) -> bool {
        self.menu_active
    }
    pub fn flip_vis(mut self) {
        self.menu_active = !self.menu_active;
    }

    // draw the menu in preparation for transferring to the plane
    pub fn construct_menu(mut self) {
        todo!();
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


    let up = vec3(0.0,1.0,0.0);
    let dimensions = vec2(1.5, 1.0);

    // let mut menu_pos: Mat4 = Mat4::from_translation(vec3(0.0,0.0,-1.0));
    // let mut menu_orient: Vec3 = vec3(0.0,0.0,1.0);

    sk.run(|sk| {
        // menu_pos = menu_pos;
        // menu_orient = menu_orient;
        let head = sk.input_head();
        let head_pos = head.position;
        //get the direction the user faces on the XZ plane (rotation around y)
        let mut head_orient = head.orientation;
        head_orient.x = 0.0;
        head_orient.z = 0.0;
        let _ = head_orient.normalize();

        // open menu on grip + menu (not system)
        if sk.input_controller_menu().contains(ButtonState::JUST_ACTIVE) && sk.input_controller(Handed::Left).grip == 1.0 {
            if launcher.active() == true {
                // set position of menu to be drawn
                let translation: Vec3 = head_pos + head_orient.mul_vec3(Vec3::NEG_Z);
                launcher.pos = Mat4::from_translation(translation);
                launcher.orient = head_orient.mul_vec3(Vec3::NEG_Z);
            } 
            launcher.flip_vis();
            
        }

        if sk.input_controller_menu().contains(ButtonState::ACTIVE) && !(sk.input_controller(Handed::Left).grip == 1.0) {
            start_steamvr(620980);
        }


        if launcher.active() {
            let plane = sk.mesh_gen_plane(dimensions, launcher.orient, up, 0, true);
            // draw menu
            sk.mesh_draw(plane, Material::UI, launcher.pos, BLUE, RenderLayer::LAYER0);
        }
        
    }, |_| {});
}

fn start_steamvr(appid:u32 ) {
    let id = appid.to_string();
    let url = String::from("steam://rungameid/")+id.as_str();
    match open::that(url){
        Ok(()) => println!("opened steam app {} successfully", appid),
        Err(_) => println!("couldn't open steam app {}", appid),
    }
}