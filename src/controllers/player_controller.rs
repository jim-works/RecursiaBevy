use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    actors::*,
    physics::JUMPABLE_GROUP,
    world::{Level, BlockId, UsableBlock, events::BlockUsedEvent}, items::{inventory::Inventory, UseItemEvent, EquipItemEvent, UnequipItemEvent, AttackItemEvent}, ui::{state::UIState, world_mouse_active},
};

use super::{Action, FrameMovement};

#[derive(Component, Default)]
pub struct RotateWithMouse {
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32,
    pub pitch_bound: f32,
    pub lock_pitch: bool,
    pub lock_yaw: bool,
}

#[derive(Component)]
pub struct FollowPlayer {}

#[derive(Component)]
pub struct PlayerActionOrigin {}

pub fn move_player(mut query: Query<(&ActionState<Action>, &mut FrameMovement), With<Player>>) {
    for (act, mut fm) in query.iter_mut() {
        let mut dv = Vec3::ZERO;
        dv.z -= if act.pressed(Action::MoveForward) {
            1.0
        } else {
            0.0
        };
        dv.z += if act.pressed(Action::MoveBack) {
            1.0
        } else {
            0.0
        };
        dv.x += if act.pressed(Action::MoveRight) {
            1.0
        } else {
            0.0
        };
        dv.x -= if act.pressed(Action::MoveLeft) {
            1.0
        } else {
            0.0
        };
        fm.0 = dv;
    }
}

//TODO: extract most of this into another system to look like move_player and do_planar_movement
pub fn jump_player(
    mut query: Query<
        (
            Entity,
            &mut Jump,
            &ActionState<Action>,
            &Collider,
            &GlobalTransform,
            &mut ExternalImpulse,
        ),
        With<Player>,
    >,
    ctx: Res<RapierContext>,
) {
    const DETECT_DIST: f32 = 0.05;
    for (entity, mut jump, act, collider, tf, mut ext_impulse) in query.iter_mut() {
        //check on ground
        let groups = QueryFilter {
            groups: Some(CollisionGroups::new(
                Group::ALL,
                Group::from_bits_truncate(JUMPABLE_GROUP),
            )),
            ..default()
        }
        .exclude_collider(entity);
        if let Some((_, _)) = ctx.cast_shape(
            tf.translation(),
            Quat::IDENTITY,
            Vec3::new(0.0, DETECT_DIST, 0.0),
            collider,
            1.0,
            groups,
        ) {
            jump.extra_jumps_remaining = jump.extra_jump_count;
            //don't use an extra jump since we are on the ground
            if act.just_pressed(Action::Jump) {
                ext_impulse.impulse.y += jump.current_height;
            }
        } else if jump.extra_jumps_remaining > 0 && act.just_pressed(Action::Jump) {
            //we aren't on the ground, so use an extra jump
            jump.extra_jumps_remaining -= 1;
            ext_impulse.impulse.y += jump.current_height;
        }
    }
}

pub fn rotate_mouse(
    mut query: Query<(&mut Transform, &mut RotateWithMouse, &ActionState<Action>)>,
    ui_state: Res<State<UIState>>
) {
    if !world_mouse_active(&ui_state.0) {
        return;
    }
    const SENSITIVITY: f32 = 0.01;
    for (mut tf, mut rotation, action) in query.iter_mut() {
        if let Some(delta) = action.axis_pair(Action::Look) {
            if !rotation.lock_yaw {
                rotation.yaw -= delta.x() * SENSITIVITY;
            }
            if !rotation.lock_pitch {
                rotation.pitch -= delta.y() * SENSITIVITY;
            }

            rotation.pitch = rotation
                .pitch
                .clamp(-rotation.pitch_bound, rotation.pitch_bound);

            tf.rotation = Quat::from_axis_angle(Vec3::Y, rotation.yaw)
                * Quat::from_axis_angle(Vec3::X, rotation.pitch)
                * Quat::from_axis_angle(Vec3::Z, rotation.roll);
        }
    }
}

pub fn follow_local_player(
    player_query: Query<(&Transform, &RotateWithMouse), With<LocalPlayer>>,
    mut follow_query: Query<
        (&mut Transform, Option<&mut RotateWithMouse>),
        (With<FollowPlayer>, Without<LocalPlayer>),
    >,
) {
    if let Ok((player_tf, player_rot)) = player_query.get_single() {
        for (mut follow_tf, opt_follow_rot) in follow_query.iter_mut() {
            follow_tf.translation = player_tf.translation + Vec3::new(0.0, 1.5, 0.0);
            if let Some(mut follow_rot) = opt_follow_rot {
                follow_rot.yaw = player_rot.yaw;
            }
        }
    }
}

pub fn player_punch(
    mut commands: Commands,
    camera_query: Query<
        (&GlobalTransform, &ActionState<Action>),
        (
            With<PlayerActionOrigin>,
            With<FollowPlayer>,
            Without<LocalPlayer>,
        ),
    >,
    player_query: Query<(Entity, &Player, &CombatInfo), With<LocalPlayer>>,
    combat_query: Query<&CombatInfo>,
    mut attack_punch_writer: EventWriter<AttackEvent>,
    mut attack_item_writer: EventWriter<AttackItemEvent>,
    level: Res<Level>,
    collision: Res<RapierContext>,
    ui_state: Res<State<UIState>>,
    id_query: Query<&BlockId>
) {
    if !world_mouse_active(&ui_state.0) {
        return;
    }
    if let Ok((tf, act)) = camera_query.get_single() {
        if act.just_pressed(Action::Punch) {
            let (player_entity, player, info) = player_query.get_single().unwrap();
            //first test if we punched a combatant
            let groups = QueryFilter {
                    groups: Some(CollisionGroups::new(
            Group::ALL,
                Group::from_bits_truncate(crate::physics::ACTOR_GROUP),
                )),
                ..default()
            }.exclude_collider(player_entity);
            if let Some((hit,_)) = collision.cast_ray(tf.translation(), tf.forward(), 10.0, true, groups) {
                if combat_query.contains(hit) {
                    //we hit a combatant, so attack it and return
                    match &info.equipped_weapon {
                        Some(weapon) => attack_item_writer.send(AttackItemEvent(player_entity, weapon.clone(), *tf)),
                        None => attack_punch_writer.send(AttackEvent { attacker: player_entity, target: hit, damage: player.hit_damage, knockback: tf.forward() })
                    };
                    return;
                }
            }
            //if not, break a block
            if let Some(hit) = level.blockcast(tf.translation(), tf.forward() * 10.0) {
                level.damage_block(hit.block_pos, 0.5, &id_query, &mut commands);
            }
        }
    }
}

pub fn player_use(
    camera_query: Query<
        (&GlobalTransform, &ActionState<Action>),
        (
            With<PlayerActionOrigin>,
            With<FollowPlayer>,
            Without<LocalPlayer>,
        ),
    >,
    player_query: Query<(&Inventory, Entity), With<LocalPlayer>>,
    mut item_use_writer: EventWriter<UseItemEvent>,
    ui_state: Res<State<UIState>>,
    level: Res<Level>,
    usable_block_query: Query<&UsableBlock>,
    mut block_use_writer: EventWriter<BlockUsedEvent>
) {
    if !world_mouse_active(&ui_state.0) {
        return;
    }
        if let Ok((inv, entity)) = player_query.get_single() {
        if let Ok((tf, act)) = camera_query.get_single() {
            if act.just_pressed(Action::Use) {
                //first test if we used a block
                if let Some(hit) = level.blockcast(tf.translation(), tf.forward() * 10.0) {
                    if level.use_block(hit.block_pos, entity, &usable_block_query, &mut block_use_writer) {
                        //we used a block, so don't also use an item
                        return;
                    }
                }
                //we didn't use a block, so try to use an item
                inv.use_item(inv.selected_slot(), *tf,&mut item_use_writer);
            }
        }
    }
}

pub fn player_scroll_inventory(
    mut query: Query<(&mut Inventory, &ActionState<Action>), With<LocalPlayer>>,
    mut equip_writer: EventWriter<EquipItemEvent>,
    mut unequip_writer: EventWriter<UnequipItemEvent>,
    ui_state: Res<State<UIState>>
) {
    if !world_mouse_active(&ui_state.0) {
        return;
    }
    const SCROLL_SENSITIVITY: f32 = 0.05;
    if let Ok((mut inv, act)) = query.get_single_mut() {
        let delta = act.value(Action::Scroll);
        let slot_diff = if delta > SCROLL_SENSITIVITY {-1} else if delta < -SCROLL_SENSITIVITY {1} else {0};
        let curr_slot = inv.selected_slot();
        inv.select_slot(curr_slot as i32 + slot_diff, &mut equip_writer, &mut unequip_writer);
    }
}