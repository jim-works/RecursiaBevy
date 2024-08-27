pub mod crosshair;
pub mod debug;
pub mod healthbar;
pub mod inventory;
pub mod main_menu;
pub mod player_stats;
pub mod state;
pub mod styles;
pub mod waves;

use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use leafwing_input_manager::action_state::ActionState;

use crate::actors::LocalPlayer;
use crate::controllers::Action;
use crate::world::LevelSystemSet;
use crate::GameState;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<state::UIState>()
            .add_systems(Startup, styles::init)
            .add_plugins((
                inventory::InventoryPlugin,
                crosshair::CrosshairPlugin,
                healthbar::HealthbarPlugin,
                player_stats::PlayerStatsUiPlugin,
                waves::WavesPlugin,
                main_menu::MainMenuPlugin,
            ))
            .add_plugins(debug::DebugUIPlugin)
            .add_systems(OnEnter(GameState::Game), state::on_load)
            .add_systems(OnEnter(state::UIState::Default), capture_mouse)
            .add_systems(OnEnter(state::UIState::Inventory), release_mouse)
            .add_systems(
                Update,
                (state::toggle_hidden, state::toggle_debug).in_set(LevelSystemSet::Main),
            )
            .add_systems(
                Update,
                (toggle_fullscreen, change_button_colors, do_button_action),
            )
            .insert_resource(UiScale(2.0));
    }
}

#[derive(Component, Clone)]
pub struct ButtonAction {
    pub action: SystemId,
    prev_state: Interaction,
}

impl ButtonAction {
    pub fn new(action: SystemId) -> Self {
        Self {
            action,
            prev_state: Interaction::default(),
        }
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
            default_background: Color::rgb_u8(70, 130, 50),
            default_border: Color::rgb_u8(37, 86, 46),
            hovered_background: Color::rgb_u8(37, 86, 46),
            hovered_border: Color::rgb_u8(25, 51, 45),
            pressed_background: Color::rgb_u8(23, 32, 56),
            pressed_border: Color::rgb_u8(37, 58, 94),
        }
    }
}

fn change_button_colors(
    mut interaction_query: Query<
        (
            &Interaction,
            &ButtonColors,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, color, mut background, mut border) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                background.0 = color.pressed_background;
                border.0 = color.pressed_border;
            }
            Interaction::Hovered => {
                background.0 = color.hovered_background;
                border.0 = color.hovered_border;
            }
            Interaction::None => {
                background.0 = color.default_background;
                border.0 = color.default_border;
            }
        }
    }
}

fn do_button_action(
    mut interaction_query: Query<
        (&Interaction, &mut ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut commands: Commands,
) {
    for (interaction, mut action) in &mut interaction_query {
        if *interaction != Interaction::Pressed && action.prev_state == Interaction::Pressed {
            commands.run_system(action.action);
        }
        action.prev_state = *interaction;
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
    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;
}

fn release_mouse(mut window_query: Query<&mut Window>) {
    let mut window = window_query.get_single_mut().unwrap();
    window.cursor.grab_mode = CursorGrabMode::None;
    window.cursor.visible = true;
}

fn toggle_fullscreen(
    mut window_query: Query<&mut Window>,
    input_query: Query<&ActionState<Action>, With<LocalPlayer>>,
) {
    if let Ok(input) = input_query.get_single() {
        if input.just_pressed(Action::ToggleFullscreen) {
            let mut window = window_query.get_single_mut().unwrap();
            window.mode = match window.mode {
                bevy::window::WindowMode::Windowed => {
                    bevy::window::WindowMode::BorderlessFullscreen
                }
                _ => bevy::window::WindowMode::Windowed,
            };
        }
    }
}
