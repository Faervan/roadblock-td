use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use enemy::EnemyPlugin;
use fastrand::Rng;
use grid::GridPlugin;
use map::MapPlugin;
use tower::TowerPlugin;
use ui::UIPlugin;

mod animation;
mod enemy;
mod grid;
mod map;
mod tower;
mod ui;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            #[cfg(debug_assertions)]
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
            ..default()
        }),
        ..default()
    }));

    if std::env::args().any(|a| a == "--egui") {
        app.add_plugins(WorldInspectorPlugin::new());
    }

    app.register_type::<Health>();

    app.insert_resource(RngResource(Rng::new()));

    app.add_plugins((
        animation::AnimationPlugin,
        GridPlugin,
        MapPlugin,
        TowerPlugin,
        EnemyPlugin,
        UIPlugin,
    ));
    app.run();
}

#[derive(Resource, Deref, DerefMut)]
struct RngResource(Rng);

#[derive(Reflect, Default, PartialEq, Debug, Clone, Copy)]
enum Orientation {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

impl Orientation {
    fn is_horizontal(&self) -> bool {
        match self {
            Orientation::Up | Orientation::Down => false,
            Orientation::Left | Orientation::Right => true,
        }
    }
}

#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
struct Health(isize);
