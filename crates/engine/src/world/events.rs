use super::{
    chunk::ChunkCoord, BlockCoord, BlockDamage, BlockId, BlockResources, Id, Level, LevelSystemSet,
};
use bevy::prelude::*;

pub struct WorldEventsPlugin;

impl Plugin for WorldEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExplosionEvent>()
            .add_event::<BlockUsedEvent>()
            .add_event::<BlockDamageSetEvent>()
            .add_event::<BlockHitEvent>()
            .add_event::<ChunkUpdatedEvent>()
            .add_systems(Update, process_explosions.in_set(LevelSystemSet::Main));
    }
}

#[derive(Event)]
pub struct BlockUsedEvent {
    pub block_position: BlockCoord,
    pub user: Entity,
    pub use_forward: Dir3,
    pub block_used: Entity,
}

#[derive(Event)]
//triggered when block gets punched
pub struct BlockHitEvent {
    pub item: Option<Entity>,
    pub user: Option<Entity>,
    pub hit_forward: Dir3,
    pub block_position: BlockCoord,
}

#[derive(Event)]
pub struct BlockDamageSetEvent {
    pub block_position: BlockCoord,
    pub damage: BlockDamage,
    pub damager: Option<Entity>,
}

#[derive(Event)]
pub struct ExplosionEvent {
    pub radius: f32,
    pub origin: BlockCoord,
}

//triggered when a chunk is spawned in or a block is changed
#[derive(Event)]
pub struct ChunkUpdatedEvent {
    pub coord: ChunkCoord,
}

fn process_explosions(
    mut reader: EventReader<ExplosionEvent>,
    level: Res<Level>,
    mut commands: Commands,
    id_query: Query<&BlockId>,
    resources: Res<BlockResources>,
    mut update_writer: EventWriter<ChunkUpdatedEvent>,
) {
    for event in reader.read() {
        let size = event.radius.ceil() as i32;
        let mut changes = Vec::with_capacity((size * size * size) as usize);
        for x in -size..size + 1 {
            for y in -size..size + 1 {
                for z in -size..size + 1 {
                    if x * x + y * y + z * z <= size * size {
                        changes.push((event.origin + BlockCoord::new(x, y, z), BlockId(Id::Empty)));
                    }
                }
            }
        }
        level.batch_set_block(
            changes.into_iter(),
            &resources.registry,
            &id_query,
            &mut update_writer,
            &mut commands,
        );
    }
}
