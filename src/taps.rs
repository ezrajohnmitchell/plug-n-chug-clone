use std::time::Duration;

use bevy::{
    app::{Plugin, Startup, Update},
    asset::Assets,
    color::Color,
    ecs::{
        component::Component,
        event::EventReader,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    math::{
        primitives::{Circle, Rectangle},
        Vec2,
    },
    render::mesh::{Mesh, Mesh2d},
    sprite::{ColorMaterial, MeshMaterial2d},
    time::{Time, Timer, Virtual},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::{
    ActiveEvents, Collider, CollisionEvent, GravityScale, RigidBody, Sensor, Velocity,
};
use rand::Rng;
use tap_state::{DrinkInput, TapState, TapStatePlugin};

use crate::{orders::Order, WINDOW_HEIGHT, WINDOW_WIDTH};

pub mod tap_state;

pub struct TapsPlugin;

impl Plugin for TapsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(TapStatePlugin);
        app.insert_resource(TapDispenseTimer(Timer::new(
            Duration::from_millis(200),
            bevy::time::TimerMode::Repeating,
        )));
        app.add_systems(Startup, add_taps);
        app.add_systems(Update, run_taps);
        app.add_systems(Update, remove_fallen_drops);
    }
}

#[derive(Resource)]
struct TapDispenseTimer(Timer);
#[derive(Component)]
pub struct Drop;

fn run_taps(
    mut commands: Commands,
    mut tap_state: ResMut<TapState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&Input, &Tap, &Transform)>,
    time: Res<Time<Virtual>>,
    mut timer: ResMut<TapDispenseTimer>,
) {
    timer.0.tick(time.delta());

    let mut rng = rand::rng();

    for (input, tap, transform) in &mut query {
        if let Some(ref mut output_state) = tap_state.get_output_state(input.0.clone()) {
            if timer.0.just_finished() && (output_state.consume_press() || output_state.on) {
                if let Some(color) = output_state.get_drop() {
                    let mut new_transform = transform.clone();
                    new_transform.translation += transform.up() * 70.;
                    new_transform.translation += -transform.forward() * 10.;
                    commands.spawn((
                        Mesh2d(meshes.add(Circle::new(3.))),
                        MeshMaterial2d(materials.add(color.clone())),
                        new_transform,
                        Drop,
                        RigidBody::Dynamic,
                        GravityScale(0.4),
                        Velocity::linear(Vec2 {
                            x: rng.random_range(-10.0..10.),
                            y: 0.
                            // y: -rng.random_range(0.0..1.),
                        }),
                        Collider::ball(3.),
                    ));
                };
            }
        }
    }
}

fn remove_fallen_drops(mut commands: Commands, mut collision_events: EventReader<CollisionEvent>) {
    for event in collision_events.read() {
        match event {
            CollisionEvent::Started(entity, entity1, collision_event_flags) => {
                commands.entity(entity1.clone()).despawn();
            },
            CollisionEvent::Stopped(entity, entity1, collision_event_flags) => {},
        }
    }
}

#[derive(Component)]
struct BarTable;

fn add_taps(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let bar_table_shape = meshes.add(Rectangle::new(800., 50.));
    let bar_table_color = Color::hsl(28., 1., 0.13);
    commands.spawn((
        BarTable,
        Mesh2d(bar_table_shape),
        MeshMaterial2d(materials.add(bar_table_color)),
        Transform::from_xyz(0., (-WINDOW_HEIGHT / 2.) + 25., 5.),
        RigidBody::Fixed,
        Collider::cuboid(400., 25.),
        ActiveEvents::COLLISION_EVENTS,
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
