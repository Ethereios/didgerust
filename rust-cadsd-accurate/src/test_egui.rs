use bevy::prelude::*;
use bevy_egui::EguiPlugin;

fn main() {
    println!("Starting Bevy + egui test...");
    
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.2)))
        .add_systems(Startup, setup)
        .add_systems(Update, ui_system);
    
    app.run();
}

fn setup(mut commands: Commands) {
    println!("Setup called - spawning camera");
    commands.spawn(Camera2d);
    println!("Camera spawned");
}

fn ui_system() {
    // Don't try to access egui context here
    // Let bevy_egui handle rendering automatically
}
