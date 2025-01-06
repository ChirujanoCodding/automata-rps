use bevy::prelude::{Entity, Resource, States};

#[derive(Resource, Default)]
pub struct GenerableRegions(pub Vec<(f32, f32, f32)>);

#[derive(Resource, Default)]
pub struct CollidablePairs(pub Vec<(Entity, Entity)>);

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    LoadingRes,
    InGame,
    Paused,
}

#[derive(Resource)]
pub struct DebugState {
    pub points: bool,
    pub rocks: bool,
    pub papers: bool,
    pub scissors: bool,
    pub radius_rocks: bool,
    pub radius_papers: bool,
    pub radius_scissors: bool,
}

#[derive(Resource)]
pub struct GameControl {
    pub stop: bool,
    pub sound: bool,
}

impl Default for GameControl {
    fn default() -> Self {
        Self {
            stop: true,
            sound: true,
        }
    }
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            points: false,
            rocks: true,
            papers: true,
            scissors: true,
            radius_rocks: false,
            radius_papers: false,
            radius_scissors: false,
        }
    }
}
