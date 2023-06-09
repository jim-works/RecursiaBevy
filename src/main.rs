//disable console window from popping up on windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::f32::consts::PI;

use actors::{Jump, Player, LocalPlayer, ActorPlugin, CombatInfo};
use bevy::{prelude::*, render::{primitives::Frustum, camera::CameraProjection}};
use bevy_fly_camera::FlyCameraPlugin;
use bevy_rapier3d::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_atmosphere::prelude::*;
use chunk_loading::{ChunkLoader, ChunkLoaderPlugin};
use controllers::{ControllersPlugin, RotateWithMouse, FollowPlayer, ControllableBundle, PlayerActionOrigin};
use leafwing_input_manager::InputManagerBundle;
use mesher::MesherPlugin;
use physics::{PhysicsPlugin, ACTOR_GROUP, PLAYER_GROUP, PhysicsObjectBundle};
use world::*;
use worldgen::WorldGenPlugin;

mod actors;
mod chunk_loading;
mod controllers;
mod mesher;
mod physics;
mod util;
mod world;
mod worldgen;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(AtmospherePlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(FlyCameraPlugin)
        .add_plugin(MesherPlugin)
        .add_plugin(WorldGenPlugin)
        .add_plugin(ChunkLoaderPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(ControllersPlugin)
        .add_plugin(ActorPlugin)
        .insert_resource(Level::new(5))
        .insert_resource(AmbientLight {
            brightness: 0.3,
            ..default()
        })
        .add_startup_system(init)
        .run();
}

fn init(mut commands: Commands, ass: Res<AssetServer>) {
    commands
        .spawn((
            
            Player {},
            LocalPlayer {},
            CombatInfo::new(10.0, 0.0),
            RotateWithMouse {
                lock_pitch: true,
                ..default()
            },
            TransformBundle::from_transform(Transform::from_xyz(-2.0, -25.0, 5.0)),
            ControllableBundle {
                physics: PhysicsObjectBundle {
                    collision_groups: CollisionGroups::new(
                    Group::from_bits_truncate(PLAYER_GROUP | ACTOR_GROUP),
                    Group::all(),
                    ),
                    ..default()
                },
                ..default()
            },
            Jump::default(),
            ChunkLoader {
                radius: 6,
                lod_levels: 2,
            },
            InputManagerBundle {
                input_map: controllers::get_input_map(),
                ..default()
            },
        ));
    commands.spawn(CombatInfo::new(10000.0,100.0));
    //todo: fix frustrum culling
    let projection = PerspectiveProjection {
        fov: PI / 2.,
        ..default()
    };
    commands.spawn((Camera3dBundle {
                transform: Transform::from_xyz(0.0,1.5,0.0),
                projection: Projection::Perspective(projection.clone()),
                frustum: Frustum::from_view_projection(&projection.get_projection_matrix()),
                ..default()
            },
            AtmosphereCamera::default(),
            FogSettings {
                color: Color::rgba(1.0, 1.0, 1.0, 0.5),
                falloff: FogFalloff::from_visibility_colors(
                    500.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
                    Color::rgba(0.35, 0.5, 0.5, 0.5), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                    Color::rgba(0.8, 0.844, 1.0, 0.5), // atmospheric inscattering color (light gained due to scattering from the sun)
                ),
                ..default()
            },

            RotateWithMouse {
                pitch_bound: PI * 0.49,
                lock_yaw: true,
                ..default()
            },
            FollowPlayer{},
            PlayerActionOrigin{},
            InputManagerBundle {
                input_map: controllers::get_input_map(),
                ..default()
            },
            ));

    //glowjelly
    let glowjelly_gltf = ass.load("glowjelly/glowjelly.gltf#Scene0");
    commands.spawn(SceneBundle {
        scene: glowjelly_gltf,
        transform: Transform::from_xyz(0.0, 10.0, 0.0).with_scale(Vec3::ONE*10.0),
        ..Default::default()
    });
}


