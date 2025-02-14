use std::time::Duration;

use bevy::{
    app::{Plugin, Update}, asset::Assets, color::Color, core::Name, ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        query::With,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut, Resource},
    }, hierarchy::{BuildChildren, ChildBuild}, math::{
        primitives::Circle,
        Vec2,
    }, render::{mesh::{Mesh, Mesh2d}, view::{InheritedVisibility, Visibility}}, sprite::{ColorMaterial, MeshMaterial2d, Sprite}, state::{
        condition::in_state,
        state::{OnEnter, OnExit},
    }, time::{Time, Timer, Virtual}, transform::{components::Transform}
};
use bevy_rapier2d::prelude::{
    ActiveEvents, Collider, CollisionEvent, GravityScale, RigidBody, Velocity,
};
use rand::Rng;
pub use tap_state::{add_tap_state, timers, DrinkInput, DrinkOutput, TapState};

use crate::{assets::BarAssets, GameStates, WINDOW_HEIGHT};

use super::{orders::OpenForOrder, GameScreen, StatePlugin};

pub mod tap_state;

pub struct TapsPlugin(GameStates);

impl Plugin for TapsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(TapDispenseTimer(Timer::new(
            Duration::from_millis(200),
            bevy::time::TimerMode::Repeating,
        )));
        app.add_systems(OnEnter(self.0.clone()), (add_tap_state, add_taps));
        app.add_systems(
            Update,
            (timers, run_taps, remove_fallen_drops).run_if(in_state(self.0.clone())),
        );
        app.add_systems(OnExit(self.0.clone()), despawn_resources);
    }
}

impl StatePlugin<TapsPlugin> for TapsPlugin {
    fn run_on_state(state: GameStates) -> TapsPlugin {
        TapsPlugin(state)
    }
}

fn despawn_resources(mut commands: Commands) {
    commands.remove_resource::<TapState>();
    commands.remove_resource::<TapDispenseTimer>();
}

#[derive(Resource)]
struct TapDispenseTimer(Timer);
#[derive(Component)]
pub struct ColorDrop(pub Color);

fn run_taps(
    mut commands: Commands,
    mut tap_state: ResMut<TapState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&Input, &Transform)>,
    time: Res<Time<Virtual>>,
    mut timer: ResMut<TapDispenseTimer>,
) {
    timer.0.tick(time.delta());

    let mut rng = rand::rng();

    for (input, transform) in &mut query {
        if let Some(ref mut output_state) = tap_state.get_output_state(input.0.clone()) {
            if timer.0.just_finished() && (output_state.consume_press() || output_state.on) {
                if let Some(color) = output_state.get_drop() {
                    let mut new_transform = transform.clone();
                    new_transform.translation += -transform.forward() * 2.;
                    commands.spawn((
                        Mesh2d(meshes.add(Circle::new(2.))),
                        MeshMaterial2d(materials.add(color.clone())),
                        new_transform,
                        ColorDrop(color.clone()),
                        RigidBody::Dynamic,
                        GravityScale(0.4),
                        Velocity::linear(Vec2 {
                            x: rng.random_range(-10.0..10.),
                            y: 0., // y: -rng.random_range(0.0..1.),
                        }),
                        Collider::ball(2.),
                        GameScreen,
                    ));
                };
            }
        }
    }
}

fn remove_fallen_drops(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    bar_table: Query<Entity, With<BarTable>>,
) {
    match bar_table.get_single() {
        Ok(bar_table) => {
            for event in collision_events.read() {
                match event {
                    CollisionEvent::Started(entity, entity1, _collision_event_flags) => {
                        if *entity == bar_table {
                            commands.entity(entity1.clone()).despawn();
                        } else if *entity1 == bar_table {
                            commands.entity(entity.clone()).despawn();
                        }
                    }
                    _ => {}
                }
            }
        }
        Err(_) => {
        }
    };
}

#[derive(Component)]
struct BarTable;

fn add_taps(
    mut commands: Commands,
    bar_assets: Res<BarAssets>
) {
    //background
    commands.spawn((
        Sprite::from_image(bar_assets.background.clone()),
        Transform::from_xyz(0., 0., -100.),
        GameScreen
    ));

    //bar table 
    commands.spawn((
        BarTable,
        Sprite::from_image(bar_assets.bar_table.clone()),
        Collider::cuboid(665. / 2., 82. / 2.),
        RigidBody::Fixed,
        Transform::from_xyz(0., -WINDOW_HEIGHT / 2. + 82. / 2., 10.),
        ActiveEvents::COLLISION_EVENTS,
        GameScreen
    ));

    //taps
    commands.spawn((
        Sprite::from_image(bar_assets.taps.clone()),
        Transform::from_xyz(0., -WINDOW_HEIGHT / 2. + 82. + 151. / 2., 5.),
        GameScreen,
        Visibility::Visible,
        Name::new("TAPS")
    )).with_children(|parent| {
        parent.spawn((
            Tap,
            Input(DrinkInput::Tap2),
            Transform::from_xyz(0., 12., 4.),
            OpenForOrder::new(),
            Name::new("TAP 2"),
            InheritedVisibility::VISIBLE
        ));
        parent.spawn((
            Tap,
            Input(DrinkInput::Tap1),
            Transform::from_xyz(-149.,12., 4.),
            OpenForOrder::new(),
            Name::new("TAP 1"),
            InheritedVisibility::VISIBLE
        ));
        parent.spawn((
            Tap,
            Input(DrinkInput::Tap3),
            Transform::from_xyz(152., 12., 4.),
            OpenForOrder::new(),
            Name::new("TAP 3"),
            InheritedVisibility::VISIBLE
        ));
    });
}

#[derive(Component, Debug, Default)]
pub struct Tap;

#[derive(Component, Debug, Default)]
struct Mixer;

#[derive(Component, Debug)]
struct Input(DrinkInput);
