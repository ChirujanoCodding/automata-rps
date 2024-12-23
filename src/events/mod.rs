use bevy::prelude::{Entity, Event};

#[derive(Event)]
pub struct DangerEvent {
    pub actor: Entity,
    pub target: Entity,
}
