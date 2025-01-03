use std::ops::Deref;

use bevy::prelude::*;

use crate::{
    entities::{Paper, Rock, Scissors},
    resources::{DebugState, GameControl, GameState, GenerableRegions},
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(DebugState::default())
            .insert_resource(GameControl::default())
            .add_systems(Startup, debug_regions)
            .add_systems(
                Update,
                (
                    toggle_view_regions,
                    toggle_rock,
                    toggle_rock_radius,
                    toggle_paper,
                    toggle_paper_radius,
                    toggle_scissors,
                    toggle_scissors_radius,
                    control_time,
                    control_sound,
                ),
            );
    }
}

#[derive(Component)]
pub struct DebugPoint;

#[derive(Bundle)]
pub struct DebugRadius {
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<ColorMaterial>,
    pub visible: Visibility,
}

// just initialize the debug points
fn debug_regions(
    regions: Res<GenerableRegions>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let dot = meshes.add(Circle::new(2.));
    let color = materials.add(Color::linear_rgb(255., 0., 0.));
    for &(x, y, r) in regions.0.deref() {
        let border = meshes.add(Annulus::new(r - 1., r + 1.));
        commands
            .spawn((
                DebugPoint,
                Mesh2d(dot.clone()),
                MeshMaterial2d(color.clone()),
                Transform::from_xyz(x, y, 100.),
                Visibility::Hidden,
            ))
            .with_children(|c| {
                c.spawn((
                    Mesh2d(border),
                    MeshMaterial2d(color.clone()),
                    Visibility::Inherited,
                ));
            });
    }
}

fn toggle_view_regions(
    mut res: ResMut<DebugState>,
    mut query: Query<&mut Visibility, With<DebugPoint>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if query.is_empty() {
        return;
    }

    if keys.just_pressed(KeyCode::KeyD) {
        res.points = !res.points;
    }

    let new_vis = res.points;

    for mut vis in query.iter_mut() {
        *vis = if new_vis {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn toggle_rock(
    mut res: ResMut<DebugState>,
    mut query: Query<&mut Visibility, With<Rock>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if query.is_empty() {
        return;
    }

    if !keys.pressed(KeyCode::ShiftLeft) && keys.just_pressed(KeyCode::Digit1) {
        res.rocks = !res.rocks;
    }

    let new_vis = res.rocks;

    for mut vis in query.iter_mut() {
        *vis = if new_vis {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn toggle_rock_radius(
    mut res: ResMut<DebugState>,
    mut query: Query<&mut Visibility>,
    rock_query: Query<&Children, With<Rock>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if query.is_empty() {
        return;
    }

    if keys.pressed(KeyCode::ShiftLeft) && keys.just_pressed(KeyCode::Digit1) {
        res.radius_rocks = !res.radius_rocks;
    }

    let new_vis = res.radius_rocks;
    let children = rock_query.iter().flatten();
    let mut iter = query.iter_many_mut(children);
    while let Some(mut vis) = iter.fetch_next() {
        *vis = if new_vis {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn toggle_paper_radius(
    mut res: ResMut<DebugState>,
    mut query: Query<&mut Visibility>,
    rock_query: Query<&Children, With<Paper>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if query.is_empty() {
        return;
    }

    if keys.pressed(KeyCode::ShiftLeft) && keys.just_pressed(KeyCode::Digit2) {
        res.radius_papers = !res.radius_papers;
    }

    let new_vis = res.radius_papers;
    let children = rock_query.iter().flatten();
    let mut iter = query.iter_many_mut(children);
    while let Some(mut vis) = iter.fetch_next() {
        *vis = if new_vis {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn toggle_paper(
    mut res: ResMut<DebugState>,
    mut query: Query<&mut Visibility, With<Paper>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if query.is_empty() {
        return;
    }

    if !keys.pressed(KeyCode::ShiftLeft) && keys.just_pressed(KeyCode::Digit2) {
        res.papers = !res.papers;
    }

    let new_vis = res.papers;

    for mut vis in query.iter_mut() {
        *vis = if new_vis {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn toggle_scissors(
    mut res: ResMut<DebugState>,
    mut query: Query<&mut Visibility, With<Scissors>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if query.is_empty() {
        return;
    }

    if !keys.pressed(KeyCode::ShiftLeft) && keys.just_pressed(KeyCode::Digit3) {
        res.scissors = !res.scissors;
    }

    let new_vis = res.scissors;

    for mut vis in query.iter_mut() {
        *vis = if new_vis {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn toggle_scissors_radius(
    mut res: ResMut<DebugState>,
    mut query: Query<&mut Visibility>,
    rock_query: Query<&Children, With<Scissors>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if query.is_empty() {
        return;
    }

    if keys.pressed(KeyCode::ShiftLeft) && keys.just_pressed(KeyCode::Digit3) {
        res.radius_scissors = !res.radius_scissors;
    }

    let new_vis = res.radius_scissors;
    let children = rock_query.iter().flatten();
    let mut iter = query.iter_many_mut(children);
    while let Some(mut vis) = iter.fetch_next() {
        *vis = if new_vis {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn control_sound(mut res: ResMut<GameControl>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyS) {
        res.sound = !res.sound;
    }
}

fn control_time(
    mut res: ResMut<GameControl>,
    mut next: ResMut<NextState<GameState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        res.stop = !res.stop;
    }
    if res.stop {
        next.set(GameState::Paused);
    } else {
        next.set(GameState::InGame);
    }
}
