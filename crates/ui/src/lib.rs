//disable console window from popping up on windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//have to enable this because it's a nursery feature
#![warn(clippy::disallowed_types)]
//bevy system signatures often violate these rules
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
//TODO: remove this before release. annoying as balls during development
#![allow(dead_code)]
#![feature(assert_matches)]
#![feature(let_chains)]

pub mod crosshair;
pub mod inventory;
pub mod main_menu;
pub mod player_stats;
pub mod state;
pub mod styles;
pub mod waves;

use bevy::picking::focus::PickingInteraction;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy_simple_text_input;
use engine::camera::MainCamera;
use leafwing_input_manager::action_state::ActionState;

use engine::controllers::Action;
use engine::world::LevelSystemSet;
use engine::GameState;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<state::UIState>()
            .add_systems(Startup, styles::init)
            .add_plugins((
                inventory::InventoryPlugin,
                crosshair::CrosshairPlugin,
                player_stats::PlayerStatsUiPlugin,
                waves::WavesPlugin,
                main_menu::MainMenuPlugin,
                bevy_simple_text_input::TextInputPlugin,
            ))
            .add_systems(OnEnter(GameState::Game), state::on_load)
            .add_systems(OnEnter(state::UIState::Default), capture_mouse)
            .add_systems(OnEnter(state::UIState::Inventory), release_mouse)
            .add_systems(Update, state::toggle_hidden.in_set(LevelSystemSet::Main))
            .add_systems(
                Update,
                (
                    toggle_fullscreen,
                    change_button_colors,
                    update_main_camera_ui,
                ),
            )
            .insert_resource(UiScale(2.0));
    }
}

#[derive(Component, Clone)]
pub struct ButtonColors {
    pub default_background: Color,
    pub default_border: Color,
    pub hovered_background: Color,
    pub hovered_border: Color,
    pub pressed_background: Color,
    pub pressed_border: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        Self {
            default_background: Color::srgb_u8(70, 130, 50),
            default_border: Color::srgb_u8(37, 86, 46),
            hovered_background: Color::srgb_u8(37, 86, 46),
            hovered_border: Color::srgb_u8(25, 51, 45),
            pressed_background: Color::srgb_u8(23, 32, 56),
            pressed_border: Color::srgb_u8(37, 58, 94),
        }
    }
}

fn change_button_colors(
    mut interaction_query: Query<
        (
            &PickingInteraction,
            &ButtonColors,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<PickingInteraction>, With<Button>),
    >,
) {
    for (interaction, color, mut background, mut border) in &mut interaction_query {
        match *interaction {
            PickingInteraction::Pressed => {
                background.0 = color.pressed_background;
                border.0 = color.pressed_border;
            }
            PickingInteraction::Hovered => {
                background.0 = color.hovered_background;
                border.0 = color.hovered_border;
            }
            PickingInteraction::None => {
                background.0 = color.default_background;
                border.0 = color.default_border;
            }
        }
    }
}

pub fn world_mouse_active(state: &state::UIState) -> bool {
    match state {
        state::UIState::Hidden => true,
        state::UIState::Default => true,
        state::UIState::Inventory => false,
    }
}

fn capture_mouse(mut window_query: Query<&mut Window>) {
    let mut window = window_query.get_single_mut().unwrap();
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
    window.cursor_options.visible = false;
}

fn release_mouse(mut window_query: Query<&mut Window>) {
    let mut window = window_query.get_single_mut().unwrap();
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
}

fn toggle_fullscreen(mut window_query: Query<&mut Window>, action: Res<ActionState<Action>>) {
    if action.just_pressed(&Action::ToggleFullscreen) {
        let mut window = window_query.get_single_mut().unwrap();
        window.mode = match window.mode {
            bevy::window::WindowMode::Windowed => {
                bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Current)
            }
            _ => bevy::window::WindowMode::Windowed,
        };
    }
}

#[derive(Component)]
pub struct MainCameraUIRoot;

fn update_main_camera_ui(
    mut commands: Commands,
    camera: Res<MainCamera>,
    ui_query: Query<Entity, With<MainCameraUIRoot>>,
) {
    for ui_element in ui_query.iter() {
        if let Some(mut ec) = commands.get_entity(ui_element) {
            ec.try_insert(TargetCamera(camera.0));
        }
    }
}
