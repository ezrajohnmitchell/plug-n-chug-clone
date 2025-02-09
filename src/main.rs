use bevy::{
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    utils::default,
    window::{PresentMode, Window, WindowPlugin, WindowTheme},
};
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use controls::ControlPlugin;
use orders::OrderPlugin;
use taps::TapsPlugin;

pub mod controls;
pub mod orders;
pub mod taps;

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
            TapsPlugin,
            OrderPlugin,
            ControlPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());
    commands.insert_resource(ClearColor(Color::hsl(183., 1., 0.5)));
}
