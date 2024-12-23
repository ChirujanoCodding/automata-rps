#![allow(clippy::type_complexity)]

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_rand::prelude::*;
use rand::Rng;
use std::fmt::Debug;

use crate::{
    entities::{ColliderType, HasEnemy, HasSprite, HasTarget, Paper, Rock, Scissors},
    events::DangerEvent,
};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DangerEvent>()
            .add_plugins(PhysicsPlugins::default())
            // .add_plugins(PhysicsDebugPlugin::default())
            .add_systems(Startup, (Self::start,))
            .add_systems(PostUpdate, Self::control_entities)
            .add_systems(
                PhysicsSchedule,
                (Self::reset_velocity).in_set(PhysicsStepSet::Last),
            )
            .add_systems(Update, Self::detect_danger)
            .add_systems(
                Update,
                (
                    // Self::control_entities,
                    Self::entity_movement::<Rock>,
                    Self::entity_movement::<Paper>,
                    Self::entity_movement::<Scissors>,
                ),
            )
            .add_systems(
                Update,
                (
                    Self::detect_collisions::<Rock>,
                    Self::detect_collisions::<Paper>,
                    Self::detect_collisions::<Scissors>,
                ),
            );
    }
}

impl GameplayPlugin {
    fn start(
        mut commands: Commands,
        mut rng: ResMut<GlobalEntropy<WyRand>>,
        query: Query<&Window>,
        server: Res<AssetServer>,
    ) {
        let window = query.single();
        let width = (window.resolution.width() - 40.) / 2.;
        let height = (window.resolution.height() - 40.) / 2.;

        std::iter::repeat_n(0, 3).for_each(|_| {
            std::iter::repeat_n(Rock, 3).for_each(|entity| {
                let width = rng.gen_range(0.0..width);
                let height = rng.gen_range(0.0..height);
                Self::spawn(&mut commands, &mut rng, &server, entity, (width, height))
            });
            std::iter::repeat_n(Paper, 3).for_each(|entity| {
                let width = rng.gen_range(0.0..width);
                let height = rng.gen_range(0.0..height);
                Self::spawn(&mut commands, &mut rng, &server, entity, (width, height))
            });
            std::iter::repeat_n(Scissors, 3).for_each(|entity| {
                let width = rng.gen_range(0.0..width);
                let height = rng.gen_range(0.0..height);
                Self::spawn(&mut commands, &mut rng, &server, entity, (width, height))
            });
        });
    }

    fn detect_danger(
        mut query: Query<(&Transform, &mut LinearVelocity)>,
        mut events: EventReader<DangerEvent>,
    ) {
        for ev in events.read() {
            let target_pos = query.get(ev.target).unwrap().0.translation;
            let (pos, mut vel) = query.get_mut(ev.actor).unwrap();

            let direction = (target_pos - pos.translation).normalize() * -75.;

            *vel = LinearVelocity(Vec2::from_array([direction.x, direction.y]));
        }
    }

    fn detect_collisions<E: Component + HasEnemy + HasTarget + HasSprite + Copy + Debug>(
        colliders: Query<(&ColliderType, &Parent)>,
        entities: Query<&E>,
        enemies: Query<&E::Enemy>,
        targets: Query<&E::Target>,
        mut commands: Commands,
        assets: Res<AssetServer>,
        mut collisions: EventReader<Collision>,
        mut events: EventWriter<DangerEvent>,
    ) {
        for Collision(contacts) in collisions.read() {
            let collider_1 = *colliders.get(contacts.entity1).unwrap().0;
            let collider_2 = *colliders.get(contacts.entity2).unwrap().0;

            match (collider_1, collider_2) {
                (ColliderType::Intern, ColliderType::Intern) => {
                    let parent_1 = colliders.get(contacts.entity1).unwrap().1;
                    let parent_2 = colliders.get(contacts.entity2).unwrap().1;

                    let (entity, target) = {
                        if let Ok(entity) = entities.get(parent_1.get()) {
                            let target = targets.get(parent_2.get());
                            ((entity, parent_1), (target, parent_2))
                        } else if let Ok(entity) = entities.get(parent_2.get()) {
                            let target = targets.get(parent_1.get());
                            ((entity, parent_2), (target, parent_1))
                        } else {
                            continue;
                        }
                    };

                    if let ((e, _), (Ok(_), t)) = (entity, target) {
                        let mut sprite = Sprite::from_image(assets.load(e.img()));
                        sprite.custom_size = Some(Vec2::splat(40.));
                        let audio_player = AudioPlayer::new(assets.load(e.sound()));
                        commands
                            .entity(t.get())
                            .remove::<(E::Target, Sprite, AudioPlayer)>()
                            .insert((*e, sprite, audio_player));
                    }
                }
                (ColliderType::Extern, ColliderType::Intern) => {
                    let parent_1 = colliders.get(contacts.entity1).unwrap().1;
                    let parent_2 = colliders.get(contacts.entity2).unwrap().1;

                    let (entity, enemy) = {
                        if let Ok(entity) = entities.get(parent_1.get()) {
                            let enemy = enemies.get(parent_2.get());
                            if enemy.is_err() {
                                continue;
                            }
                            ((entity, parent_1), (enemy, parent_2))
                        } else if let Ok(entity) = entities.get(parent_2.get()) {
                            let enemy = enemies.get(parent_1.get());
                            if enemy.is_err() {
                                continue;
                            }
                            ((entity, parent_2), (enemy, parent_1))
                        } else {
                            continue;
                        }
                    };

                    if let ((_, e), (Ok(_), t)) = (entity, enemy) {
                        events.send(DangerEvent {
                            actor: e.get(),
                            target: t.get(),
                        });
                    }
                }
                _ => continue,
            };
        }
    }

    fn reset_velocity(mut query: Query<&mut LinearVelocity>) {
        for mut vel in query.iter_mut() {
            *vel = LinearVelocity(Vec2::ZERO);
        }
    }

    fn control_entities(
        mut query: Query<(&mut Transform, &mut LinearVelocity)>,
        window_query: Query<&Window>,
    ) {
        let window = window_query.single();
        let (half_width, half_height) = (
            (window.resolution.width() - 40.) / 2.,
            (window.resolution.height() - 40.) / 2.,
        );

        for (mut pos, mut vel) in query.iter_mut() {
            let translation = &mut pos.translation;

            // Control en el eje X
            if translation.x.abs() >= half_width {
                let direction = translation.x.signum();
                translation.x = direction * (half_width - 0.1); // Reposicionar dentro del límite
                vel.0.x *= -1.; // Invertir velocidad
                translation.x += -direction * 1.0; // Mover ligeramente hacia el interior
            }

            // Control en el eje Y
            if translation.y.abs() >= half_height {
                let direction = translation.y.signum();
                translation.y = direction * (half_height - 0.1); // Reposicionar dentro del límite
                vel.0.y *= -1.; // Invertir velocidad
                translation.y += -direction * 1.0; // Mover ligeramente hacia el interior
            }
        }
    }

    fn entity_movement<E: Component + HasEnemy + HasTarget>(
        mut rng: ResMut<GlobalEntropy<WyRand>>,
        mut query: Query<(&Transform, &mut LinearVelocity), With<E>>,
        targets: Query<(&Transform, Entity), (Without<E>, With<E::Target>)>,
    ) {
        for (pos, mut vel) in query.iter_mut() {
            let closest_target = targets.iter().min_by_key(|(t, _)| {
                Vec2::new(
                    pos.translation.x - t.translation.x,
                    pos.translation.y - t.translation.y,
                )
                .length() as i32
            });

            if let Some((t, _)) = closest_target {
                let direction = (t.translation - pos.translation).normalize() * 5.;
                let random_x = direction.x + rng.gen_range(-50.0..50.);
                let random_y = direction.y + rng.gen_range(-50.0..50.);

                vel.x += random_x;
                vel.y += random_y;
            } else {
                continue;
            };
        }
    }

    fn spawn<T: Component + HasEnemy + HasTarget + HasSprite>(
        commands: &mut Commands,
        rng: &mut ResMut<GlobalEntropy<WyRand>>,
        server: &Res<AssetServer>,
        entity: T,
        (width, height): (f32, f32),
    ) {
        let x = rng.gen_range(-width..width);
        let y = rng.gen_range(-height..height);

        let radius = rng.gen_range(55.0..150.0);

        let mut sprite = Sprite::from_image(server.load(entity.img()));
        sprite.custom_size = Some(Vec2::splat(40.));

        let mut cmd = commands.spawn((
            entity,
            sprite,
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            Transform::from_xyz(x, y, 0.0),
            LinearVelocity(Vec2::splat(25.)),
            SweptCcd::LINEAR,
        ));
        cmd.with_children(|children| {
            children
                .spawn(Collider::rectangle(26., 26.))
                .insert(ColliderType::Intern);

            children
                .spawn(Collider::circle(radius))
                .insert(ColliderType::Extern)
                .insert(Sensor);
        });
    }
}
