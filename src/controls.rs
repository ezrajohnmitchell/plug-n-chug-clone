use bevy::{
    app::{Plugin, Update},
    ecs::system::{Res, ResMut, Resource},
    input::{keyboard::KeyCode, ButtonInput},
};

use crate::taps::tap_state::{DrinkInput, DrinkOutput, TapState};

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(SelectedTap(Option::None));
        app.add_systems(Update, control_system);
    }
}

#[derive(Resource)]
pub struct SelectedTap(Option<DrinkOutput>);

/// temporary keyboard controls for development
fn control_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut tap_state: ResMut<TapState>,
    mut selected_tap: ResMut<SelectedTap>,
) {
    if selected_tap.0 == Option::None {
        if keys.just_pressed(KeyCode::Numpad1) {
            selected_tap.0 = Option::Some(DrinkOutput::Color1);
        }
        if keys.just_pressed(KeyCode::Numpad2) {
            selected_tap.0 = Option::Some(DrinkOutput::Color2);
        }
        if keys.just_pressed(KeyCode::Numpad3) {
            selected_tap.0 = Option::Some(DrinkOutput::Color3);
        }
        if keys.just_pressed(KeyCode::Numpad4) {
            selected_tap.0 = Option::Some(DrinkOutput::Mixer1);
        }
        if keys.just_pressed(KeyCode::Numpad5) {
            selected_tap.0 = Option::Some(DrinkOutput::Mixer2);
        }
    } else {
        let output = selected_tap.0.as_ref().unwrap();
        if keys.just_pressed(KeyCode::Numpad1) {
            tap_state.make_connection(output.clone(), DrinkInput::Tap1);
            selected_tap.0 = Option::None;
        } else if keys.just_pressed(KeyCode::Numpad2) {
            tap_state.make_connection(output.clone(), DrinkInput::Tap2);
            selected_tap.0 = Option::None;
        } else if keys.just_pressed(KeyCode::Numpad3) {
            tap_state.make_connection(output.clone(), DrinkInput::Tap3);
            selected_tap.0 = Option::None;
        } else if keys.just_pressed(KeyCode::Numpad4) {
            tap_state.make_connection(output.clone(), DrinkInput::Mixer1);
            selected_tap.0 = Option::None;
        } else if keys.just_pressed(KeyCode::Numpad5) {
            tap_state.make_connection(output.clone(), DrinkInput::Mixer2);
            selected_tap.0 = Option::None;
        } else if keys.just_pressed(KeyCode::Numpad0){
            tap_state.disconnect(output.clone());
            selected_tap.0 = Option::None;
        } else if keys.just_pressed(KeyCode::ArrowUp) {
            tap_state.output_switch(true, output.clone());
            selected_tap.0 = Option::None;
        } else if keys.just_pressed(KeyCode::ArrowDown) {
            tap_state.output_switch(false, output.clone());
            selected_tap.0 = Option::None;
        } else if keys.just_pressed(KeyCode::ArrowRight) {
            tap_state.mixer_switch(true, output.clone());
            selected_tap.0 = Option::None;
        } else if keys.just_pressed(KeyCode::ArrowLeft) {
            tap_state.mixer_switch(false, output.clone());
            selected_tap.0 = Option::None;
        } else if keys.just_pressed(KeyCode::Space) {
            tap_state.drop_pressed(output.clone());
            selected_tap.0 = Option::None;
        }
    }
}
