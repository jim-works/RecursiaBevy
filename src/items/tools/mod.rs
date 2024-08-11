use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    actors::Player,
    physics::{collision::Aabb, query},
    world::{
        events::{BlockDamageSetEvent, BlockHitEvent, ChunkUpdatedEvent},
        BlockId, BlockPhysics, Level,
    },
};

use super::{
    inventory::Inventory, CreatorItem, EquipItemEvent, HitResult, ItemStack, ItemSystemSet,
    MaxStackSize, PickupItemEvent, SwingEndEvent, SwingItemEvent,
};

pub mod abilities;

pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(abilities::ToolAbilitiesPlugin)
            .register_type::<Tool>()
            .register_type::<ToolResistance>()
            .register_type::<DontHitBlocks>()
            .add_systems(
                Update,
                (on_swing, deal_block_damage).in_set(ItemSystemSet::UsageProcessing),
            );
    }
}

//denotes required power if attached to a block
//is used in `Tool` to give power of said tool
#[derive(Copy, Clone, Hash, Eq, Debug, PartialEq, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub enum ToolResistance {
    Instant, //instantly broken
    Axe(u32),
    Pickaxe(u32),
    Shovel(u32),
}

impl Default for ToolResistance {
    fn default() -> Self {
        ToolResistance::Pickaxe(0)
    }
}

#[derive(
    Copy, Clone, Hash, Eq, Debug, PartialEq, Component, Reflect, Default, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct Tool {
    pub axe: u32,
    pub pickaxe: u32,
    pub shovel: u32,
}

#[derive(
    Copy, Clone, Hash, Eq, Debug, PartialEq, Component, Reflect, Default, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct DontHitBlocks;

pub fn on_swing(
    mut reader: EventReader<SwingItemEvent>,
    mut writer: EventWriter<BlockHitEvent>,
    mut swing_hit_writer: EventWriter<SwingEndEvent>,
    level: Res<Level>,
    block_physics_query: Query<&BlockPhysics>,
    object_query: Query<(Entity, &GlobalTransform, &Aabb)>,
    item_query: Query<(), Without<DontHitBlocks>>,
) {
    for SwingItemEvent {
        user,
        inventory_slot,
        stack,
        tf,
    } in reader.read()
    {
        if !item_query.contains(stack.id) {
            continue;
        }
        if let Some(query::RaycastHit::Block(block_position, hit)) = query::raycast(
            query::Raycast::new(tf.translation, tf.forward(), 10.0),
            &level,
            &block_physics_query,
            &object_query,
            &[*user],
        ) {
            writer.send(BlockHitEvent {
                item: Some(stack.id),
                user: Some(*user),
                block_position,
                hit_forward: tf.forward(),
            });
            swing_hit_writer.send(SwingEndEvent {
                user: *user,
                inventory_slot: *inventory_slot,
                stack: *stack,
                result: HitResult::Hit(hit.hit_pos),
            })
        } else {
            swing_hit_writer.send(SwingEndEvent {
                user: *user,
                inventory_slot: *inventory_slot,
                stack: *stack,
                result: HitResult::Miss,
            })
        }
    }
}

fn deal_block_damage(
    mut reader: EventReader<BlockHitEvent>,
    resistance_query: Query<&ToolResistance>,
    tool_query: Query<&Tool>,
    level: Res<Level>,
    id_query: Query<&BlockId>,
    mut player_query: Query<&mut Inventory, With<Player>>,
    block_query: Query<&CreatorItem>,
    data_query: Query<&MaxStackSize>,
    mut pickup_writer: EventWriter<PickupItemEvent>,
    mut equip_writer: EventWriter<EquipItemEvent>,
    mut writer: EventWriter<BlockDamageSetEvent>,
    mut update_writer: EventWriter<ChunkUpdatedEvent>,
    mut commands: Commands,
) {
    for BlockHitEvent {
        item,
        user,
        block_position,
        hit_forward: _,
    } in reader.read()
    {
        if let Some(block_hit) = level.get_block_entity(*block_position) {
            let resistance = resistance_query.get(block_hit).copied().unwrap_or_default();
            let tool = item
                .and_then(|i| tool_query.get(i).ok())
                .copied() //block was hit with a tool, so use that
                .unwrap_or(
                    user.and_then(|entity| tool_query.get(entity).ok())
                        .copied() //entity that hit the block had tool power
                        .unwrap_or_default(),
                ); //...or nothing and use default tool
                   //todo - remove this once item drops are entities
            if let Some(broken) = level.damage_block(
                *block_position,
                calc_block_damage(resistance, tool),
                *item,
                &id_query,
                &mut writer,
                &mut update_writer,
                &mut commands,
            ) {
                if let Some(mut inv) = user.and_then(|e| player_query.get_mut(e).ok()) {
                    if let Ok(CreatorItem(item)) = block_query.get(broken) {
                        inv.pickup_item(
                            ItemStack::new(*item, 1),
                            &data_query,
                            &mut pickup_writer,
                            &mut equip_writer,
                        );
                    }
                }
            }
        }
    }
}

pub fn calc_block_damage(resistance: ToolResistance, tool: Tool) -> f32 {
    const MAX_HITS_TO_BREAK: u32 = 5;
    match resistance {
        ToolResistance::Instant => 1.0,
        ToolResistance::Axe(required) => {
            if required <= tool.axe {
                1.0 / (MAX_HITS_TO_BREAK.saturating_sub(tool.axe.saturating_sub(required)) as f32)
                    .max(1.0)
            } else {
                0.0
            }
        }
        ToolResistance::Pickaxe(required) => {
            if required <= tool.pickaxe {
                1.0 / (MAX_HITS_TO_BREAK.saturating_sub(tool.pickaxe.saturating_sub(required))
                    as f32)
                    .max(1.0)
            } else {
                0.0
            }
        }
        ToolResistance::Shovel(required) => {
            if required <= tool.shovel {
                1.0 / (MAX_HITS_TO_BREAK.saturating_sub(tool.shovel.saturating_sub(required))
                    as f32)
                    .max(1.0)
            } else {
                0.0
            }
        }
    }
}
