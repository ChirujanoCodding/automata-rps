use bevy::prelude::{Resource, States};

#[derive(Resource, Default)]
pub struct GenerableRegions(pub Vec<(f32, f32, f32)>);

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

#[derive(Resource, Default)]
pub struct GameControl {
    pub stop: bool,
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
