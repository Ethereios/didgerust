use bevy::prelude::*;

fn main() {
    println!("Starting minimal Bevy test...");
    
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    println!("Setup called - spawning camera");
    commands.spawn(Camera2d);
    println!("Camera spawned");
}
