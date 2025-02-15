use bevy::{app::Plugin, ecs::{component::Component, system::ResMut}, state::{app::AppExtStates, state::{NextState, OnEnter, OnExit, StateSet, SubStates}}};
use controls::ControlPlugin;
use orders::OrderPlugin;
use taps::TapsPlugin;

use crate::{despawn_screen, GameStates};

pub mod controls;
pub mod orders;
pub mod taps;
pub mod status_bar;

pub struct GamePlugin(GameStates);

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            TapsPlugin::run_on_state(self.0.clone()),
            OrderPlugin::run_on_state(self.0.clone()),
            ControlPlugin::run_on_state(self.0.clone()),
        ));
        app.add_systems(OnExit(self.0.clone()), despawn_screen::<GameScreen>);
        app.add_sub_state::<LevelState>();
        app.add_systems(OnEnter(LevelState::GameOver), end_game);
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

pub enum Event{
    FailedOrder
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)] 
#[source(GameStates = GameStates::Playing)]
pub enum LevelState {
    #[default]
    NoFailures,
    OrdersFailed(usize),
    GameOver
}

impl LevelState {
    pub fn next(&self, event: &Event) -> LevelState{
        use Event::*;
        use LevelState::*;

        match (self, event){
            (NoFailures, FailedOrder) => OrdersFailed(0),
            (OrdersFailed(val), FailedOrder) => {
                if val + 1 >= 3 {
                    return GameOver;
                }
                OrdersFailed(val + 1)
            },
            (GameOver, FailedOrder) => GameOver,
        }
    }
}

fn end_game(mut next_state: ResMut<NextState<GameStates>>){
    next_state.set(GameStates::EndScreen);
}