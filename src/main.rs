use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, init)
        .run();
}

fn init(mut commands: Commands) {
    commands.spawn(Camera2d);
}
