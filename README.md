# OXRLauncher-rs

An OpenXR Launcher built in rust for VR and XR games and other utilities 

## Build
```bash
cargo build --release
```
## Usage
```bash
cargo run --release
```

the keybind for opening the menu is holding grip and menu buttons on the left hand with palm facing towards the headset. This behaviour is because stereokit-rs doesnt expose the system button (which it shouldn't) and will be addressed in later version when possible.