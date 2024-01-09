use bevy::prelude::*;
use big_brain::prelude::*;

use crate::{
    actors::{
        ai::{scorers::AggroScorer, AttackAction, WalkToCurrentTargetAction},
        AggroTargets, AggroPlayer,
    },
    controllers::ControllableBundle,
    physics::{PhysicsBundle, GRAVITY, collision::Aabb, FrictionBundle, movement::Velocity},
    util::{physics::aim_projectile_straight_fallback, plugin::SmoothLookTo, SendEventCommand},
};

use super::{
    coin::SpawnCoinEvent, world_anchor::WorldAnchor, ActorName, ActorResources, CombatInfo,
    CombatantBundle, Damage, DefaultAnimation, Jump, MoveSpeed, UninitializedActor,
};

#[derive(Resource)]
pub struct SkeletonPirateResources {
    pub scene: Handle<Scene>,
    pub anim: Handle<AnimationClip>,
}

#[derive(Component, Default)]
pub struct SkeletonPirate {
    scene: Option<Entity>,
}

#[derive(Component)]
pub struct SkeletonPirateScene;

#[derive(Event)]
pub struct SpawnSkeletonPirateEvent {
    pub location: GlobalTransform,
}

pub struct SkeletonPiratePlugin;

impl Plugin for SkeletonPiratePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (load_resources, add_to_registry))
            .add_systems(PreUpdate, attack.in_set(BigBrainSet::Actions))
            .add_systems(Update, spawn_skeleton_pirate)
            .add_event::<SpawnSkeletonPirateEvent>();
    }
}

pub fn load_resources(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(SkeletonPirateResources {
        scene: assets.load("skeletons/skeleton_pirate.gltf#Scene0"),
        anim: assets.load("skeletons/skeleton_pirate.gltf#Animation0"),
    });
}

fn add_to_registry(mut res: ResMut<ActorResources>) {
    res.registry.add_dynamic(
        ActorName::core("skeleton_pirate"),
        Box::new(|commands, tf| {
            commands.add(SendEventCommand(SpawnSkeletonPirateEvent { location: tf }))
        }),
    );
}

pub fn spawn_skeleton_pirate(
    mut commands: Commands,
    skele_res: Res<SkeletonPirateResources>,
    mut spawn_requests: EventReader<SpawnSkeletonPirateEvent>,
    anchor: Query<Entity, With<WorldAnchor>>,
) {
    let anchor_entity = anchor.get_single().ok().unwrap_or(Entity::PLACEHOLDER);
    const ATTACK_RANGE: f32 = 10.0;
    const AGGRO_RANGE: f32 = ATTACK_RANGE*2.0 + 5.0;
    for spawn in spawn_requests.read() {
        commands.spawn((
            SceneBundle {
                scene: skele_res.scene.clone_weak(),
                transform: spawn.location.compute_transform(),
                ..default()
            },
            Name::new("SkeletonPirate"),
            CombatantBundle {
                combat_info: CombatInfo::new(10.0, 0.0),
                ..default()
            },
            PhysicsBundle {
                collider: Aabb::new(Vec3::new(0.8,1.6,0.8), Vec3::ZERO),
                ..default()
            },
            FrictionBundle::default(),
            ControllableBundle {
                jump: Jump::new(12.0, 0),
                move_speed: MoveSpeed::new(50.0, 10.0, 5.0),
                ..default()
            },
            SmoothLookTo::new(0.7),
            SkeletonPirate { ..default() },
            AggroPlayer { range: AGGRO_RANGE, priority: 0 },
            AggroTargets::new(vec![(anchor_entity, i32::MIN)]),
            DefaultAnimation::new(Handle::default(), Entity::PLACEHOLDER, 0.5, 1.0),
            UninitializedActor,
            Thinker::build()
                .label("skeleton thinker")
                .picker(Highest)
                .when(
                    FixedScore::build(0.01),
                    WalkToCurrentTargetAction {
                        stop_distance: ATTACK_RANGE * 0.5,
                        ..default()
                    },
                )
                .when(
                    AggroScorer {
                        range: ATTACK_RANGE,
                    },
                    AttackAction,
                ),
        ));
    }
}

//TODO: extract into separate function. all scenes should have this setup
pub fn setup_skeleton_pirate(
    mut commands: Commands,
    children_query: Query<&Children>,
    skele_res: Res<SkeletonPirateResources>,
    mut skeleton_pirate_query: Query<(Entity, &mut SkeletonPirate), With<UninitializedActor>>,
    anim_query: Query<&AnimationPlayer>,
    animations: Res<Assets<AnimationClip>>,
) {
    for (parent_id, mut skele) in skeleton_pirate_query.iter_mut() {
        //hierarchy is parent -> scene -> gltfnode (with animation player)
        //find first child with a child that has an animation player
        //we have to wait until the children get spawned in (aka the gltf loaded)
        if let Ok(children) = children_query.get(parent_id) {
            for child in children {
                let mut found = false;
                if let Ok(grandchildren) = children_query.get(*child) {
                    for candidate_anim_player in grandchildren {
                        if anim_query.contains(*candidate_anim_player) {
                            commands
                                .entity(*candidate_anim_player)
                                .insert(SkeletonPirateScene);
                            commands
                                .entity(parent_id)
                                .remove::<UninitializedActor>()
                                .insert(DefaultAnimation::new(
                                    skele_res.anim.clone(),
                                    *candidate_anim_player,
                                    1.1,
                                    if let Some(clip) = animations.get(&skele_res.anim) {
                                        clip.duration()
                                    } else {
                                        0.0
                                    },
                                ));
                            skele.scene = Some(*candidate_anim_player);

                            found = true;
                            break;
                        } else {
                            error!("SkeletonPirate animation player not found");
                        }
                    }
                }
                if found {
                    break;
                }
            }
        }
    }
}

//todo - extract into generic ranged attack system
fn attack(
    mut action_query: Query<(&Actor, &mut ActionState), With<AttackAction>>,
    mut skele_query: Query<
        (
            &GlobalTransform,
            &AggroTargets,
            &Velocity,
            Option<&mut DefaultAnimation>,
        ),
        With<SkeletonPirate>,
    >,
    aggro_query: Query<(&GlobalTransform, Option<&Velocity>)>,
    mut spawn_coin: EventWriter<SpawnCoinEvent>,
    time: Res<Time>,
) {
    const COIN_OFFSET: Vec3 = Vec3::new(0.0, 2.0, 0.0);
    const THROW_IMPULSE: f32 = 25.0;
    let combat = CombatantBundle::default();
    let damage = Damage { amount: 1.0 };
    for (&Actor(actor), mut state) in action_query.iter_mut() {
        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
                if let Ok((_, _, _, Some(mut anim))) = skele_query.get_mut(actor) {
                    anim.reset();
                }
            }
            ActionState::Executing => {
                if let Ok((tf, targets, v, anim_opt)) = skele_query.get_mut(actor) {
                    if let Some((target, target_v_opt)) = targets
                        .current_target()
                        .and_then(|t| aggro_query.get(t).ok())
                    {
                        let spawn_point = tf.translation() + COIN_OFFSET;
                        match anim_opt {
                            Some(mut anim) => {
                                anim.tick(time.delta_seconds());
                                if anim.just_acted() {
                                    spawn_coin.send(SpawnCoinEvent {
                                        location: Transform::from_translation(spawn_point),
                                        velocity: Velocity(aim_projectile_straight_fallback(
                                            target.translation() - spawn_point,
                                            target_v_opt.unwrap_or(&Velocity::default()).0
                                                - v.0,
                                            THROW_IMPULSE,
                                            GRAVITY,
                                        )),
                                        combat: combat.clone(),
                                        owner: actor,
                                        damage,
                                    });
                                }
                                if anim.finished() {
                                    *state = ActionState::Success;
                                }
                                continue;
                            }
                            None => {
                                //no anim so gogogogogogogogogogogogo
                                spawn_coin.send(SpawnCoinEvent {
                                    location: Transform::from_translation(spawn_point),
                                    velocity: Velocity(aim_projectile_straight_fallback(
                                        target.translation() - spawn_point,
                                        target_v_opt.unwrap_or(&Velocity::default()).0
                                            - v.0,
                                        THROW_IMPULSE,
                                        GRAVITY,
                                    )),
                                    combat: combat.clone(),
                                    owner: actor,
                                    damage,
                                });
                                *state = ActionState::Success;
                                continue;
                            }
                        }
                    }
                }
                *state = ActionState::Success;
            }
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
