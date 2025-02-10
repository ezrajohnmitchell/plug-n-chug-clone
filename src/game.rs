use bevy::{app::Plugin, ecs::component::Component, state::state::OnExit};
use controls::ControlPlugin;
use orders::OrderPlugin;
use taps::TapsPlugin;

use crate::{despawn_screen, GameStates};

pub mod controls;
pub mod orders;
pub mod taps;

pub struct GamePlugin(GameStates);

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            TapsPlugin::run_on_state(self.0.clone()),
            OrderPlugin::run_on_state(self.0.clone()),
            ControlPlugin::run_on_state(self.0.clone()),
        ));
        app.add_systems(OnExit(self.0.clone()), despawn_screen::<GameScreen>);
    }
}

impl StatePlugin<GamePlugin> for GamePlugin {
    fn run_on_state(state: GameStates) -> GamePlugin {
        GamePlugin(state)
    }
}

pub trait StatePlugin<T: Plugin> {
    fn run_on_state(state: GameStates) -> T;

    fn despawn_resources() {}
}

#[derive(Component)]
pub struct GameScreen;
