use std::fmt::Debug;

use bevy::{math::Vec2, prelude::Component};

use crate::add_components;

#[derive(Debug)]
pub enum Mock {
    Rock,
    Paper,
    Scissors,
}

#[derive(Component, Clone, Debug, PartialEq, Eq, Copy)]
pub struct Rock;

#[derive(Component, Clone, Debug, PartialEq, Eq, Copy)]
pub struct Paper;

#[derive(Component, Clone, Debug, PartialEq, Eq, Copy)]
pub struct Scissors;

#[derive(Component)]
pub struct Enemy<T: Component>(pub T);

#[derive(Component)]
pub struct Target<T: Component>(pub T);

#[derive(Component, Clone)]
pub struct Vision(pub f32);

#[derive(Component)]
pub struct Velocity(pub Vec2);

pub trait HasEnemy {
    type Enemy: Component + Debug;
}

pub trait HasTarget {
    type Target: Component + Debug;
}

pub trait HasSprite {
    const PREFIX: &str = "sprites/";
    fn img(&self) -> String;
    fn sound(&self) -> String;
}

add_components!(Rock, Paper, Scissors);
add_components!(Paper, Scissors, Rock);
add_components!(Scissors, Rock, Paper);

impl HasSprite for Rock {
    fn img(&self) -> String {
        format!("{}/rock.png", Self::PREFIX)
    }

    fn sound(&self) -> String {
        "sounds/rock.ogg".to_owned()
    }
}

impl HasSprite for Paper {
    fn img(&self) -> String {
        format!("{}/paper.png", Self::PREFIX)
    }
    fn sound(&self) -> String {
        "sounds/paper.ogg".to_owned()
    }
}

impl HasSprite for Scissors {
    fn img(&self) -> String {
        format!("{}/scissors.png", Self::PREFIX)
    }
    fn sound(&self) -> String {
        "sounds/scissors.ogg".to_owned()
    }
}
