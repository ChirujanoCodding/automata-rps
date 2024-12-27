#![allow(clippy::type_complexity)]

use std::time::Duration;
use std::{f32::consts::PI, ops::Deref};

use bevy::math::vec3;
use bevy::prelude::ops::{cos, sin};
use bevy::{math::vec2, prelude::*};
use bevy_rand::prelude::*;
use bevy_spatial::kdtree::KDTree2;
use bevy_spatial::{AutomaticUpdate, SpatialAccess, SpatialStructure};
use rand::Rng;

use crate::constants::{SPEED_FACTOR, SPRITE_SIZE};
use crate::entities::{Velocity, Vision};
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
                resolution: (1280., 720.).into(),
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
        .init_state::<GameState>()
        .insert_resource(GenerableRegions::default())
        .add_systems(
            Startup,
            (setup, spawn_entities)
                .chain()
                .run_if(in_state(GameState::LoadingRes)),
        )
        .add_systems(
            Update,
            (
                move_entity::<Rock>,
                move_entity::<Paper>,
                move_entity::<Scissors>,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            PostUpdate,
            check_boundaries.run_if(in_state(GameState::InGame)),
        );
    }
}

type KdTree<T> = KDTree2<T>;

fn move_entity<T: Component + HasTarget + HasEnemy + HasSprite + Copy>(
    time: Res<Time>,
    mut commands: Commands,
    server: Res<AssetServer>,
    mut query: Query<(&mut Transform, &Velocity, &Vision, &T)>,
    mut target_query: Query<Entity, (With<T::Target>, Without<T>)>,
    target_tree: Res<KdTree<T::Target>>,
    enemies_tree: Res<KdTree<T::Enemy>>,
) {
    if query.is_empty() {
        return;
    }

    for (mut entity, velocity, vision, me) in query.iter_mut() {
        let pos = entity.translation.xy() + velocity.0 * SPEED_FACTOR * time.delta_secs();
        entity.translation = vec3(pos.x, pos.y, 0.0);

        if let Some((target_pos, Some(target))) = target_tree.nearest_neighbour(pos) {
            if target_query.is_empty() {
                continue;
            };
            let towards = (target_pos - pos).normalize();
            let distance = pos.distance(target_pos);
            if distance > SPRITE_SIZE * 2. {
                entity.translation += vec3(towards.x, towards.y, 0.0);
            } else {
                let mut sprite = Sprite::from_image(server.load(me.img()));
                sprite.custom_size = Some(Vec2::splat(SPRITE_SIZE * 2.));

                if let Ok(target) = target_query.get_mut(target) {
                    commands
                        .entity(target)
                        .remove::<(T::Target, Sprite)>()
                        .insert((*me, sprite));
                }
            }
        }

        let within_distance = enemies_tree.within_distance(pos, vision.0);

        let nearest_enemy = within_distance.iter().reduce(|acc, e| {
            let closest = (acc.0 - pos).length_squared();
            let current = (e.0 - pos).length_squared();
            if closest < current {
                acc
            } else {
                e
            }
        });

        if let Some(&(enemy_pos, Some(_))) = nearest_enemy {
            let push = (enemy_pos - pos).normalize() * SPEED_FACTOR * -5.;
            entity.translation += vec3(push.x, push.y, 0.0);
        }
    }
}

fn check_boundaries(window: Query<&Window>, mut query: Query<(&Transform, &mut Velocity)>) {
    if window.is_empty() || query.is_empty() {
        return;
    };

    let window = window.get_single().unwrap();
    let width = (window.width() - 38.) / 2.;
    let height = (window.height() - 38.) / 2.;

    for (entity, mut velocity) in query.iter_mut() {
        let pos = entity.translation.xy();

        if pos.x <= -width || pos.x >= width {
            velocity.0.x = -velocity.0.x;
        }

        if pos.y <= -height || pos.y >= height {
            velocity.0.y = -velocity.0.y;
        }
    }
}

fn setup(mut regions: ResMut<GenerableRegions>, query: Query<&Window>) {
    let window = query.single();
    let width = window.resolution.width() / 2.;
    let height = window.resolution.height() / 2.;
    let generated_regions = generate_regions(width, height, 12);
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
        std::iter::repeat_n((), 25).for_each(|_| {
            let angle = rng.gen_range(0.0..(2.0 * PI));
            let pos = vec2(x, y) + vec2(cos(angle), sin(angle)) * rng.gen_range(0.0..r);
            let transform = Transform::from_xyz(pos.x, pos.y, 0.0);

            match i % len {
                0 => spawn(
                    &mut commands,
                    Rock,
                    transform,
                    &server,
                    &mut meshes,
                    material.clone(),
                ),
                1 => spawn(
                    &mut commands,
                    Paper,
                    transform,
                    &server,
                    &mut meshes,
                    material.clone(),
                ),
                2 => spawn(
                    &mut commands,
                    Scissors,
                    transform,
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
    server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: MeshMaterial2d<ColorMaterial>,
) {
    let mut sprite = Sprite::from_image(server.load(entity.img()));
    sprite.custom_size = Some(Vec2::splat(SPRITE_SIZE * 2.));
    let velocity = Velocity(Vec2::splat(1.));
    let r = SPRITE_SIZE + 50.;
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
