use std::path::PathBuf;
use glam::*;
use stereokit::{
    *,
    named_colors::*
};

fn main() {
    let sk = crate::SettingsBuilder::new()
        .app_name("OpenXRLauncher")
        .init()
        .unwrap();
    let mut menu_active: bool = false;
    let up = vec3(0.0,1.0,0.0);
    // let normal = vec3(0.0,0.0,1.0);
    let dimensions = vec2(1.5, 1.0);
    // let position = vec3(0.0,0.0,-1.0);
    // let transform = Mat4::from_translation(position);

    let cube = sk.mesh_gen_cube(vec3(1.0, 1.0,1.0), 0);

    sk.run(|sk| {
        // open menu on grip + menu (not system)
        if sk.input_controller_menu().contains(ButtonState::JUST_ACTIVE) && sk.input_controller(Handed::Left).grip == 1.0 {
            if menu_active == true {
                menu_active = false;
            } else {
                menu_active = true;
            }
            // draw menu
        }

        sk.mesh_draw(&cube, Material::UI, Mat4::from_translation(vec3(0.0,0.0,-1.0)), WHITE, RenderLayer::LAYER0);
        if menu_active {
            let head = sk.input_head();
            let head_pos = head.position;
            //get the direction the user faces on the XZ plane (rotation around y)
            let mut head_orient = head.orientation;
            head_orient.x = 0.0;
            head_orient.z = 0.0;
            let _ = head_orient.normalize();

            let translation: Vec3 = head_pos + head_orient.mul_vec3(Vec3::NEG_Z);
            let menu_pos: Mat4 = Mat4::from_translation(translation);

            // let theta: f32 = PI-(head_orient.y/head_orient.w).atan();
            // let menu_quat = quat(0, head_orient.y, 0, head_orient.w).normalize();
            let menu_orient: Vec3 = head_orient.mul_vec3(Vec3::NEG_Z);

            let plane = sk.mesh_gen_plane(dimensions, menu_orient, up, 0, true);
            // draw menu
            sk.mesh_draw(plane, Material::UI, menu_pos, BLUE, RenderLayer::LAYER0);
        }
        
    }, |_| {});
}
