#![allow(clippy::type_complexity)]

use std::time::Duration;
use std::{f32::consts::PI, ops::Deref};

use bevy::math::vec3;
use bevy::prelude::ops::{cos, sin};
use bevy::{math::vec2, prelude::*};
use bevy_kira_audio::*;
use bevy_rand::prelude::*;
use bevy_spatial::kdtree::KDTree2;
use bevy_spatial::{AutomaticUpdate, SpatialAccess, SpatialStructure};
use rand::Rng;

use crate::constants::{SPEED_FACTOR, SPRITE_SIZE};
use crate::entities::{Velocity, Vision};
use crate::resources::CollidablePairs;
use crate::{
    entities::{HasEnemy, HasSprite, HasTarget, Paper, Rock, Scissors},
    resources::{GameState, GenerableRegions},
    utils::generate_regions,
};

use super::debug::{DebugPlugin, DebugRadius};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (1920., 1080.).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(
            AutomaticUpdate::<Rock>::new()
                .with_spatial_ds(SpatialStructure::KDTree2)
                .with_frequency(Duration::from_millis(1)),
        )
        .add_plugins(
            AutomaticUpdate::<Paper>::new()
                .with_spatial_ds(SpatialStructure::KDTree2)
                .with_frequency(Duration::from_millis(1)),
        )
        .add_plugins(
            AutomaticUpdate::<Scissors>::new()
                .with_spatial_ds(SpatialStructure::KDTree2)
                .with_frequency(Duration::from_millis(1)),
        )
        .add_plugins(DebugPlugin)
        .add_plugins(AudioPlugin)
        .init_state::<GameState>()
        .insert_resource(GenerableRegions::default())
        .insert_resource(CollidablePairs::default())
        .add_systems(
            Startup,
            (
                setup,
                spawn_entities,
                // detect_collisions::<Rock>,
                // detect_collisions::<Paper>,
                // detect_collisions::<Scissors>,
                // resolve_collisions,
            )
                .chain()
                .run_if(in_state(GameState::LoadingRes)),
        )
        .add_systems(
            Update,
            (
                handle_targets::<Rock>,
                handle_enemies::<Rock>,
                handle_targets::<Paper>,
                handle_enemies::<Paper>,
                handle_targets::<Scissors>,
                handle_enemies::<Scissors>,
                detect_collisions::<Rock>,
                detect_collisions::<Paper>,
                detect_collisions::<Scissors>,
                update_positions,
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
        // .add_systems(
        //     Update,
        //     (
        //         detect_collisions::<Rock>,
        //         detect_collisions::<Paper>,
        //         detect_collisions::<Scissors>,
        //     )
        //         .run_if(in_state(GameState::InGame)),
        // )
        .add_systems(
            PostUpdate,
            (
                check_boundaries,
                resolve_collisions::<Rock>,
                resolve_collisions::<Paper>,
                resolve_collisions::<Scissors>,
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}

type KdTree<T> = KDTree2<T>;

fn detect_collisions<T: Component>(
    query: Query<(Entity, &Transform)>,
    tree: Res<KdTree<T>>,
    mut collision_pairs: ResMut<CollidablePairs>,
) {
    collision_pairs.0.clear();

    for (entity, transform) in query.iter() {
        let pos = transform.translation.xy();

        let nearby = tree.within_distance(pos, SPRITE_SIZE * 2.);

        for &(_, other_entity) in nearby.iter() {
            if let Some(other_entity) = other_entity {
                if entity != other_entity {
                    collision_pairs.0.push((entity, other_entity));
                }
            }
        }
    }
}

fn resolve_collisions<T: Component>(
    mut query: Query<&mut Transform, With<T>>,
    collision_pairs: Res<CollidablePairs>,
) {
    let estimated_distance = SPRITE_SIZE * 2.;
    if query.is_empty() {
        return;
    };
    for &(entity_a, entity_b) in collision_pairs.0.iter() {
        println!("{:?} - {:?}", entity_a, entity_b);
        // match query.get_many_mut([entity_a, entity_b]) {
        //     Ok(_) => {
        //         println!("success")
        //     }
        //     Err(e) => println!("{e:?}"),
        // };
        let Ok([mut transform_a, mut transform_b]) = query.get_many_mut([entity_a, entity_b])
        else {
            continue;
        };

        let pos_a = transform_a.translation.xy();
        let pos_b = transform_b.translation.xy();

        let distance = pos_a.distance(pos_b);
        if distance < estimated_distance {
            let separation = (pos_b - pos_a).normalize() * (estimated_distance - distance) * 1.1;

            transform_a.translation -= vec3(separation.x, separation.y, 0.0);
            transform_b.translation += vec3(separation.x, separation.y, 0.0);
        }
    }
}

fn update_positions(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    if query.is_empty() {
        return;
    }

    for (mut transform, velocity) in query.iter_mut() {
        let pos = transform.translation.xy() + velocity.0 * SPEED_FACTOR * time.delta_secs();
        transform.translation = vec3(pos.x, pos.y, 0.0);
    }
}

fn handle_targets<T: Component + HasTarget + HasSprite + Copy>(
    mut commands: Commands,
    server: Res<AssetServer>,
    audio: Res<Audio>,
    mut query: Query<(&mut Transform, &T)>,
    mut targets: Query<Entity, (With<T::Target>, Without<T>)>,
    tree: Res<KdTree<T::Target>>,
) {
    if query.is_empty() {
        return;
    }

    for (mut transform, me) in query.iter_mut() {
        let pos = transform.translation.xy();

        if let Some((target_pos, Some(target))) = tree.nearest_neighbour(pos) {
            if targets.is_empty() {
                continue;
            }
            let dis = pos.distance(target_pos);

            if dis <= SPRITE_SIZE * 2. {
                let mut sprite = Sprite::from_image(server.load(me.img()));
                sprite.custom_size = Some(Vec2::splat(SPRITE_SIZE * 2.));
                let sound = server.load(me.sound());
                audio.play(sound);

                if let Ok(target) = targets.get_mut(target) {
                    commands
                        .entity(target)
                        .remove::<(T::Target, Sprite)>()
                        .insert((*me, sprite));
                }
            } else {
                let towards = (target_pos - pos).normalize();
                transform.translation += vec3(towards.x, towards.y, 0.0);
            }
        }
    }
}

fn handle_enemies<T: Component + HasEnemy>(
    mut query: Query<(&mut Transform, &Vision), With<T>>,
    tree: Res<KdTree<T::Enemy>>,
) {
    for (mut transform, vision) in query.iter_mut() {
        let pos = transform.translation.xy();

        let within_distance = tree.within_distance(pos, vision.0);
        let nearest_enemy = within_distance.iter().reduce(|acc, e| {
            let closest = (acc.0 - pos).length_squared();
            let current = (e.0 - pos).length_squared();
            if closest < current {
                acc
            } else {
                e
            }
        });

        if let Some(&(enemy_pos, _)) = nearest_enemy {
            let push = (enemy_pos - pos).normalize() * SPEED_FACTOR * -5.;
            transform.translation += vec3(push.x, push.y, 0.0);
        }
    }
}

fn check_boundaries(window: Query<&Window>, mut query: Query<(&mut Transform, &mut Velocity)>) {
    if window.is_empty() || query.is_empty() {
        return;
    }

    let window = window.get_single().unwrap();
    let width = (window.width() - 38.) / 2.;
    let height = (window.height() - 38.) / 2.;
    let gap = 5.0; // Adjust this value for the desired gap size

    for (mut entity, mut velocity) in query.iter_mut() {
        let mut pos = entity.translation.xy();

        // Check for left boundary
        if pos.x <= -width + gap {
            pos.x = -width + gap;
            velocity.0.x = velocity.0.x.abs(); // Ensure positive velocity to move away from the boundary
        }

        // Check for right boundary
        if pos.x >= width - gap {
            pos.x = width - gap;
            velocity.0.x = -velocity.0.x.abs(); // Ensure negative velocity to move away from the boundary
        }

        // Check for bottom boundary
        if pos.y <= -height + gap {
            pos.y = -height + gap;
            velocity.0.y = velocity.0.y.abs();
        }

        // Check for top boundary
        if pos.y >= height - gap {
            pos.y = height - gap;
            velocity.0.y = -velocity.0.y.abs();
        }

        // Update entity's position
        entity.translation.x = pos.x;
        entity.translation.y = pos.y;
    }
}

fn setup(mut regions: ResMut<GenerableRegions>, query: Query<&Window>) {
    let window = query.single();
    let width = window.resolution.width() / 2.;
    let height = window.resolution.height() / 2.;
    let generated_regions = generate_regions(width, height, 3);
    regions.0 = generated_regions;
}

fn spawn_entities(
    regions: Res<GenerableRegions>,
    server: Res<AssetServer>,
    mut next: ResMut<NextState<GameState>>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let len = 3;
    let regions = regions.0.deref().iter().enumerate();

    let material = MeshMaterial2d(materials.add(Color::linear_rgb(255., 0., 0.)));

    for (i, &(x, y, r)) in regions {
        std::iter::repeat_n((), 10).for_each(|_| {
            let angle = rng.gen_range(0.0..(2.0 * PI));
            let pos = vec2(x, y) + vec2(cos(angle), sin(angle)) * rng.gen_range(0.0..r);
            let transform = Transform::from_xyz(pos.x, pos.y, 0.0);
            let radius = rng.gen_range(50.0..150.0);
            match i % len {
                0 => spawn(
                    &mut commands,
                    Rock,
                    transform,
                    radius,
                    &server,
                    &mut meshes,
                    material.clone(),
                ),
                1 => spawn(
                    &mut commands,
                    Paper,
                    transform,
                    radius,
                    &server,
                    &mut meshes,
                    material.clone(),
                ),
                2 => spawn(
                    &mut commands,
                    Scissors,
                    transform,
                    radius,
                    &server,
                    &mut meshes,
                    material.clone(),
                ),
                _ => {}
            };
        });
    }

    next.set(GameState::InGame);
}

fn spawn<T: Component + HasSprite>(
    commands: &mut Commands,
    entity: T,
    transform: Transform,
    radius: f32,
    server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: MeshMaterial2d<ColorMaterial>,
) {
    let mut sprite = Sprite::from_image(server.load(entity.img()));
    sprite.custom_size = Some(Vec2::splat(SPRITE_SIZE * 2.));
    let velocity = Velocity(Vec2::splat(1.));
    let r = SPRITE_SIZE + radius;
    let vision = Vision(r);
    let mesh = meshes.add(Annulus::new(r - 1., r + 1.));
    commands
        .spawn((
            entity,
            sprite,
            transform,
            vision.clone(),
            velocity,
            Visibility::Visible,
        ))
        .with_children(|c| {
            c.spawn(DebugRadius {
                mesh: Mesh2d(mesh),
                material,
                visible: Visibility::Hidden,
            });
        });
}
