pub mod entities;
pub mod events;
pub mod plugins;
pub mod utils;

use bevy::prelude::*;
use bevy_rand::{plugin::EntropyPlugin, prelude::WyRand};
use plugins::game::GameplayPlugin;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new()
        .add_plugins(GameplayPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .insert_resource(ClearColor(Color::Srgba(Srgba::rgb(240.0, 240.0, 240.0)))) // background
        .add_systems(Startup, setup)
        .run();
}
