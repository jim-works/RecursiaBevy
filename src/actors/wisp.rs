use bevy::prelude::*;

use crate::{
    physics::{collision::Aabb, movement::GravityMult, PhysicsBundle},
    util::{plugin::SmoothLookTo, SendEventCommand},
    world::LevelLoadState,
};

use super::{ActorName, ActorResources, CombatInfo, CombatantBundle, Idler};

#[derive(Resource)]
pub struct WispResources {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

#[derive(Component, Default)]
pub struct Wisp;

#[derive(Event)]
pub struct SpawnWispEvent {
    pub location: Transform,
}

pub struct WispPlugin;

impl Plugin for WispPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (load_resources, add_to_registry))
            .add_systems(OnEnter(LevelLoadState::Loaded), trigger_spawning)
            .add_systems(Update, spawn_wisp)
            .add_event::<SpawnWispEvent>();
    }
}

pub fn load_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(WispResources {
        mesh: meshes.add(shape::Box::new(1.0,1.0,1.0).into()),
        material: materials.add(StandardMaterial::from(Color::WHITE)),
    });
}

fn trigger_spawning(mut writer: EventWriter<SpawnWispEvent>) {
    for i in 0..0 {
        writer.send(SpawnWispEvent {
            location: Transform::from_xyz(
                (i % 5) as f32 * -5.0,
                (i / 5) as f32 * 5.0 + 50.0,
                (i / 5) as f32 * -1.0 + 10.0,
            ),
        });
    }
}

fn add_to_registry(mut res: ResMut<ActorResources>) {
    res.registry.add_dynamic(
        ActorName::core("wisp"),
        Box::new(|commands, tf| commands.add(SendEventCommand(SpawnWispEvent { location: tf }))),
    );
}

fn spawn_wisp(
    mut commands: Commands,
    res: Res<WispResources>,
    mut spawn_requests: EventReader<SpawnWispEvent>,
) {
    for spawn in spawn_requests.read() {
        commands.spawn((
            PbrBundle {
                mesh: res.mesh.clone(),
                material: res.material.clone(),
                transform: spawn.location,
                ..default()
            },
            Name::new("wisp"),
            CombatantBundle {
                combat_info: CombatInfo {
                    knockback_multiplier: 2.0,
                    ..CombatInfo::new(10.0, 0.0)
                },
                ..default()
            },
            PhysicsBundle {
                collider: Aabb::centered(Vec3::splat(1.0)),
                gravity: GravityMult(0.5),
                ..default()
            },
            Wisp,
            Idler::default(),
            SmoothLookTo::new(0.5),
            bevy::pbr::CubemapVisibleEntities::default(),
            bevy::render::primitives::CubemapFrusta::default(),
        ));
    }
}
