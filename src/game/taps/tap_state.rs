use std::{
    collections::{HashMap, VecDeque},
    time::Duration,
};

use bevy::{
    color::{Color, Luminance},
    ecs::{
        component::Component,
        system::{Commands, Res, ResMut, Resource},
    },
    time::{Time, Timer, Virtual},
};

pub const COLOR_RED: Color = Color::hsl(0.0, 1.0, 0.5);
pub const COLOR_BLUE: Color = Color::hsl(232.0, 1.0, 0.5);
pub const COLOR_YELLOW: Color = Color::hsl(65.0, 1.0, 0.5);

const MAX_PENDING_DROPS: usize = 2;

pub fn add_tap_state(mut commands: Commands) {
    commands.insert_resource(TapState::new());
}

pub fn timers(time: Res<Time<Virtual>>, mut tap_state: ResMut<TapState>) {
    tap_state.tick(&time);
}

#[derive(Resource, Debug)]
pub struct TapState {
    connections: HashMap<DrinkOutput, Option<DrinkInput>>,
    outputs: HashMap<DrinkOutput, OutputState>,
    speed: usize,
}

impl TapState {
    pub fn new() -> TapState {
        let mut connections = HashMap::with_capacity(5);
        connections.insert(DrinkOutput::Color1, Option::None);
        connections.insert(DrinkOutput::Color2, Option::None);
        connections.insert(DrinkOutput::Color3, Option::None);
        connections.insert(DrinkOutput::Mixer1, Option::None);
        connections.insert(DrinkOutput::Mixer2, Option::None);

        let mut outputs = HashMap::with_capacity(5);
        outputs.insert(
            DrinkOutput::Color1,
            OutputState::new_color(COLOR_RED.clone()),
        ); //red
        outputs.insert(
            DrinkOutput::Color2,
            OutputState::new_color(COLOR_BLUE.clone()),
        ); //blue
        outputs.insert(
            DrinkOutput::Color3,
            OutputState::new_color(COLOR_YELLOW.clone()),
        ); //yellow
        outputs.insert(DrinkOutput::Mixer1, OutputState::new_mixer());
        outputs.insert(DrinkOutput::Mixer2, OutputState::new_mixer());

        TapState {
            connections,
            outputs,
            speed: 1,
        }
    }

    pub fn make_connection(&mut self, output: DrinkOutput, input: DrinkInput) {
        for (output, input_option) in self.connections.iter_mut() {
            if let Some(stored_input) = input_option {
                if *stored_input == input {
                    *input_option = Option::None;
                }
            }
        }
        self.connections
            .insert(output.clone(), Option::Some(input.clone()));
    }

    pub fn disconnect(&mut self, output: DrinkOutput) {
        self.connections.insert(output, Option::None);
    }

    pub fn drop_pressed(&mut self, output: DrinkOutput) {
        self.outputs.entry(output).and_modify(|output_state| {
            if output_state.pending_presses < MAX_PENDING_DROPS {
                output_state.pending_presses += 1
            }
        });
    }

    pub fn output_switch(&mut self, switch_on: bool, output: DrinkOutput) {
        self.outputs.entry(output).and_modify(|output_state| {
            output_state.on = switch_on;
        });
    }

    pub fn mixer_switch(&mut self, switch_on: bool, output: DrinkOutput) {
        self.outputs
            .entry(output)
            .and_modify(|output_state| match output_state.output_type {
                OutputType::Color(_) => {}
                OutputType::Mixer(ref mut mixer_output_state) => {
                    mixer_output_state.mixer_on = switch_on;
                }
            });
    }

    pub fn tick(&mut self, time: &Time<Virtual>) {
        for output_state in self.outputs.values_mut() {
            output_state.tick(time);
        }
    }

    pub fn get_output_state(&mut self, input: DrinkInput) -> Option<&mut OutputState> {
        for (output, input_option) in self.connections.iter_mut() {
            if let Some(stored_input) = input_option {
                if input == *stored_input {
                    return self.outputs.get_mut(&output.clone());
                }
            }
        }
        Option::None
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum DrinkInput {
    Tap1,
    Tap2,
    Tap3,
    Mixer1,
    Mixer2,
}

#[derive(Debug, PartialEq, Eq, Hash, Component, Clone)]
pub enum DrinkOutput {
    Color1,
    Color2,
    Color3,
    Mixer1,
    Mixer2,
}

#[derive(Debug)]
pub struct OutputState {
    pending_presses: usize,
    press_available_on: Timer,
    pub on: bool,
    output_type: OutputType,
}

impl OutputState {
    fn new_color(color: Color) -> OutputState {
        OutputState {
            on: false,
            press_available_on: Timer::new(Duration::from_millis(250), bevy::time::TimerMode::Once),
            pending_presses: 0,
            output_type: OutputType::Color(ColorOutputState::new(color)),
        }
    }

    fn new_mixer() -> OutputState {
        OutputState {
            on: false,
            press_available_on: Timer::new(Duration::from_millis(250), bevy::time::TimerMode::Once),
            pending_presses: 0,
            output_type: OutputType::Mixer(MixerOutputState::new()),
        }
    }

    pub fn drop_pressed(&mut self) {
        self.pending_presses += 1;
        if self.pending_presses > 3 {
            self.pending_presses = 3;
        }
    }

    pub fn pending_press(&self) -> bool {
        self.pending_presses > 0
    }

    /// returns true if press was available or false if press is still waiting on timer
    pub fn consume_press(&mut self) -> bool {
        if self.press_available_on.finished() && self.pending_presses > 0 {
            self.press_available_on.reset();
            self.pending_presses -= 1;
            return true;
        }

        false
    }

    pub fn tick(&mut self, time: &Time<Virtual>) {
        self.press_available_on.tick(time.delta());
    }

    pub fn set_lightness(&mut self, lightness: f32) {
        match self.output_type {
            OutputType::Color(ref mut output_state) => {
                output_state.light = lightness;
            }
            _ => {}
        }
    }

    pub fn set_mixer(&mut self, switch_on: bool) {
        match self.output_type {
            OutputType::Mixer(ref mut mixer_state) => {
                mixer_state.mixer_on = switch_on;
            }
            _ => {}
        }
    }

    pub fn get_drop(&mut self) -> Option<Color> {
        match &mut self.output_type {
            OutputType::Color(color_output_state) => Option::Some(
                color_output_state
                    .start_color
                    .clone()
                    .lighter(color_output_state.light),
            ),
            OutputType::Mixer(mixer_output_state) => mixer_output_state.mixer.pop_front(),
        }
    }
}

#[derive(Debug)]
enum OutputType {
    Color(ColorOutputState),
    Mixer(MixerOutputState),
}

#[derive(Debug)]
pub struct ColorOutputState {
    pub start_color: Color,
    pub light: f32,
}

impl ColorOutputState {
    fn new(color: Color) -> ColorOutputState {
        ColorOutputState {
            start_color: color,
            light: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct MixerOutputState {
    mixer: VecDeque<Color>,
    mixer_on: bool,
}

impl MixerOutputState {
    fn new() -> MixerOutputState {
        MixerOutputState {
            mixer_on: false,
            mixer: VecDeque::with_capacity(64),
        }
    }
}
