mod mesher;
pub use mesher::*;

mod mesh_lod;

use bevy::prelude::*;

use crate::world::LevelSystemSet;

pub struct MesherPlugin;

const SPAWN_MESH_TIME_BUDGET_COUNT: u32 = 1000;


impl Plugin for MesherPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(MeshTimer{timer: Timer::from_seconds(0.25, TimerMode::Repeating)})
            .add_systems((mesher::poll_mesh_queue,mesher::queue_meshing,mesh_lod::queue_meshing_lod).in_set(LevelSystemSet::Main));
    }
}