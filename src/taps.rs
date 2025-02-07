use bevy::{
    app::{Plugin, Startup},
    asset::Assets,
    color::Color,
    ecs::{
        component::Component,
        system::{Commands, ResMut},
    },
    math::primitives::Rectangle,
    render::mesh::{Mesh, Mesh2d},
    sprite::{ColorMaterial, MeshMaterial2d},
    transform::components::Transform,
};
use tap_state::{DrinkInput, TapStatePlugin};

use crate::{orders::Order, WINDOW_HEIGHT, WINDOW_WIDTH};

pub mod tap_state;

pub struct TapsPlugin;

impl Plugin for TapsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(TapStatePlugin);
        app.add_systems(Startup, add_taps);
    }
}

fn add_taps(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let bar_table_shape = meshes.add(Rectangle::new(800., 50.));
    let bar_table_color = Color::hsl(28., 1., 0.13);
    commands.spawn((
        Mesh2d(bar_table_shape),
        MeshMaterial2d(materials.add(bar_table_color)),
        Transform::from_xyz(0., (-WINDOW_HEIGHT / 2.) + 25., 5.),
    ));

    const TAP_HEIGHT: f32 = 150.;
    let tap_shape = meshes.add(Rectangle::new(50., TAP_HEIGHT));
    let tap_color = materials.add(Color::hsl(28., 0., 0.43));

    let x_dist = WINDOW_WIDTH / 3.;

    commands.spawn((
        Tap::default(),
        Input(DrinkInput::Tap1),
        Mesh2d(tap_shape.clone()),
        MeshMaterial2d(tap_color.clone()),
        Transform::from_xyz(-x_dist, WINDOW_HEIGHT / -2. + TAP_HEIGHT / 2. + 50., 6.),
    ));
    commands.spawn((
        Tap::default(),
        Input(DrinkInput::Tap2),
        Mesh2d(tap_shape.clone()),
        MeshMaterial2d(tap_color.clone()),
        Transform::from_xyz(0., WINDOW_HEIGHT / -2. + TAP_HEIGHT / 2. + 50., 6.),
    ));
    commands.spawn((
        Tap::default(),
        Input(DrinkInput::Tap3),
        Mesh2d(tap_shape.clone()),
        MeshMaterial2d(tap_color.clone()),
        Transform::from_xyz(x_dist, WINDOW_HEIGHT / -2. + TAP_HEIGHT / 2. + 50., 6.),
    ));

    commands.spawn((Mixer, Input(DrinkInput::Mixer1)));
    commands.spawn((Mixer, Input(DrinkInput::Mixer2)));
}

#[derive(Component, Debug)]
pub struct Tap(Option<Order>);

impl Default for Tap {
    fn default() -> Self {
        Self(Option::None)
    }
}

#[derive(Component, Debug, Default)]
struct Mixer;

#[derive(Component, Debug)]
struct Input(DrinkInput);
