use bevy::prelude::*;

use crate::{world::{Level, BlockCoord, BlockResources, BlockName, BlockId, events::ChunkUpdatedEvent, BlockType, BlockPhysics}, physics::raycast::{RaycastHit, Ray, self}};

use super::UseItemEvent;

#[derive(Component)]
pub struct BlockItem(pub Entity);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MegaBlockItem(pub BlockName, pub i32);

pub fn use_block_entity_item(
    mut reader: EventReader<UseItemEvent>,
    block_query: Query<&BlockItem>,
    level: Res<Level>,
    mut update_writer: EventWriter<ChunkUpdatedEvent>,
    block_physics_query: Query<&BlockPhysics>,
    id_query: Query<&BlockId>,
    mut commands: Commands,
) {
    for UseItemEvent { user: _, inventory_slot: _, stack, tf } in reader.read() {
        if let Ok(block_item) = block_query.get(stack.id) {
            if let Some(RaycastHit::Block(coord, hit_point, _)) = raycast::raycast(
                Ray::new(tf.translation(), tf.forward(), 10.0),
                &level,
                &block_physics_query,
            ) {
                let normal = crate::util::max_component_norm(hit_point - coord.center()).into();
                level.set_block_entity(coord+normal, BlockType::Filled(block_item.0), &id_query, &mut update_writer, &mut commands);
            }
        }
    }
}

pub fn use_mega_block_item(
    mut reader: EventReader<UseItemEvent>,
    megablock_query: Query<&MegaBlockItem>,
    level: Res<Level>,
    resources: Res<BlockResources>,
    id_query: Query<&BlockId>,
    block_physics_query: Query<&BlockPhysics>,
    mut update_writer: EventWriter<ChunkUpdatedEvent>,
    mut commands: Commands,
) {
    for UseItemEvent { user: _, inventory_slot: _, stack, tf } in reader.read() {
        if let Ok(block_item) = megablock_query.get(stack.id) {
            let id = resources.registry.get_id(&block_item.0);
            let size = block_item.1;
            if let Some(RaycastHit::Block(coord, _, _)) = raycast::raycast(
                Ray::new(tf.translation(), tf.forward(), 10.0),
                &level,
                &block_physics_query,
            ) {
                let mut changes = Vec::with_capacity((size*size*size) as usize);
                for x in -size..size+1 {
                    for y in -size..size+1 {
                        for z in -size..size+1 {
                            changes.push((
                                coord + BlockCoord::new(x, y, z),
                                id,
                            ));
                        }
                    }
                }
                level.batch_set_block(changes.into_iter(), &resources.registry, &id_query, &mut update_writer, &mut commands);
            }
        }
    }
}