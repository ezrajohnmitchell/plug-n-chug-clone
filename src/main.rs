use assets::{AssetInitializerPlugin, BarAssets, OrderAssets};
use bevy::{
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    utils::default,
    window::{PresentMode, Window, WindowPlugin, WindowTheme},
};
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState, LoadingStateAppExt,
};
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use game::{GamePlugin, StatePlugin};

pub mod assets;
pub mod game;

pub const WINDOW_WIDTH: f32 = 800.;
pub const WINDOW_HEIGHT: f32 = 400.;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "plug-n-chug".into(),
                        name: Some("plug-n-chug.app".into()),
                        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                        present_mode: PresentMode::AutoVsync,
                        // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                        prevent_default_event_handling: false,
                        window_theme: Some(WindowTheme::Dark),
                        enabled_buttons: bevy::window::EnabledButtons {
                            maximize: false,
                            ..Default::default()
                        },
                        resizable: false,
                        // mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                        visible: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        ..default()
                    }),
                    ..default()
                }),
            // LogDiagnosticsPlugin::default(),
            // FrameTimeDiagnosticsPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(200.),
            // WorldInspectorPlugin::new(),
            // RapierDebugRenderPlugin::default(),
            AssetInitializerPlugin,
            GamePlugin::run_on_state(GameStates::Playing),
        ))
        .init_state::<GameStates>()
        .add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<OrderAssets>()
                .load_collection::<BarAssets>(),
        )
        .add_systems(Startup, setup)
        .run();
}
#[derive(Resource)]
pub struct Score(usize);

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d::default(), Msaa::Sample2));
    commands.insert_resource(ClearColor(Color::hsl(183., 1., 0.5)));
    commands.insert_resource(Score(0));
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameStates {
    #[default]
    AssetLoading,
    StartMenu,
    Playing,
    EndScreen,
}

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}
