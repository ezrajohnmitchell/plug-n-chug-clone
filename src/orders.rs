use std::time::Duration;

use bevy::{
    app::{Plugin, Startup, Update},
    asset::{Assets, Handle},
    color::{Alpha, Color, Luminance},
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        query::With,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    hierarchy::{BuildChildren, ChildBuild, DespawnRecursiveExt, Parent},
    math::primitives::Rectangle,
    render::{
        mesh::{Mesh, Mesh2d},
        view::Visibility,
    },
    sprite::{ColorMaterial, MeshMaterial2d},
    time::{Time, Timer},
    transform::components::Transform,
};
use bevy_rapier2d::{
    prelude::{ActiveEvents, Collider, CollisionEvent, RigidBody, Sensor},
    rapier::prelude::CollisionEventFlags,
};
use rand::seq::{IndexedRandom, IteratorRandom};

use crate::taps::{
    tap_state::{COLOR_BLUE, COLOR_RED, COLOR_YELLOW},
    ColorDrop, Tap, TAP_HEIGHT, TAP_WIDTH,
};

pub struct OrderPlugin;

impl Plugin for OrderPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, setup_orders);
        app.add_systems(Startup, setup_cup_meshes);
        app.add_systems(Update, spawn_orders);
        app.add_systems(Update, assign_pending_orders);
        app.add_systems(Update, add_drops_to_cups);
    }
}

#[derive(Component, Debug, Clone)]
pub struct PendingOrder(Order);

#[derive(Component, Debug, Clone)]
pub struct Order {
    sections: [Color; 20], //treat 0 as buttom of the cup
    recieved: Vec<Color>,
    time_remaining: Timer,
}

#[derive(Resource)]
pub struct AvailableOrders(Vec<[Color; 20]>);

pub fn setup_orders(mut commands: Commands) {
    let mut orders: Vec<[Color; 20]> = Vec::new();

    let colors = [COLOR_RED, COLOR_BLUE, COLOR_YELLOW];

    for color in colors {
        for color2 in colors {
            let mut order_arr = [COLOR_RED; 20];
            for i in 0..10 {
                order_arr[i] = color;
            }
            for i in 10..20 {
                order_arr[i] = color2;
            }
            orders.push(order_arr);
        }
    }

    commands.insert_resource(AvailableOrders(orders));
    commands.insert_resource(OrderSpawnTimer(Timer::new(
        Duration::from_secs(3),
        bevy::time::TimerMode::Repeating,
    )));
}

#[derive(Resource)]
pub struct OrderSpawnTimer(Timer);

fn spawn_orders(
    mut commands: Commands,
    available_orders: Res<AvailableOrders>,
    mut order_timer: ResMut<OrderSpawnTimer>,
    time: Res<Time>,
) {
    order_timer.0.tick(time.delta());
    if order_timer.0.just_finished() {
        let order_to_spawn = available_orders.0.choose(&mut rand::rng());
        if let Some(order) = order_to_spawn {
            commands.spawn(PendingOrder(Order {
                sections: order.clone(),
                recieved: Vec::new(),
                time_remaining: Timer::new(Duration::from_secs(30), bevy::time::TimerMode::Once),
            }));
        }
    }
}

#[derive(Component)]
pub struct Cup;

#[derive(Component)]
pub struct CupDivider;

#[derive(Component)]
pub struct CupWall;

#[derive(Resource)]
struct CupMeshes {
    cup_wall_mesh: Handle<Mesh>,
    cup_base_mesh: Handle<Mesh>,
    cup_material: Handle<ColorMaterial>,
    divider_mesh: Handle<Mesh>,
    divider_material: Handle<ColorMaterial>,
}

const CUP_HEIGHT: f32 = TAP_HEIGHT * 0.8;
const CUP_WIDTH: f32 = TAP_WIDTH * 1.8;
const CUP_THICKNESS: f32 = 10.0;

pub fn setup_cup_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(CupMeshes {
        cup_wall_mesh: meshes.add(Rectangle::new(CUP_THICKNESS, CUP_HEIGHT)),
        cup_base_mesh: meshes.add(Rectangle::new(
            CUP_WIDTH - (CUP_THICKNESS * 2.),
            CUP_THICKNESS,
        )),
        cup_material: materials.add(Color::BLACK),
        divider_mesh: meshes.add(Rectangle::new(
            CUP_WIDTH - (CUP_THICKNESS * 2.),
            CUP_THICKNESS / 2.,
        )),
        divider_material: materials.add(Color::BLACK.lighter(0.4).with_alpha(0.8)),
    });
}

#[derive(Component)]
struct CupFillCollider;

#[derive(Component)]
pub struct OpenForOrder(Timer);
impl OpenForOrder {
    pub fn new() -> OpenForOrder{
        OpenForOrder(Timer::new(Duration::from_secs(2), bevy::time::TimerMode::Once))
    }
}

fn assign_pending_orders(
    mut commands: Commands,
    pending_orders: Query<(Entity, &PendingOrder)>,
    mut taps: Query<(Entity, &mut OpenForOrder), With<Tap>>,
    mesh_handles: Res<CupMeshes>,
    time: Res<Time>
) {
    let mut pending_orders = pending_orders.iter().choose_multiple(&mut rand::rng(), pending_orders.iter().len()).into_iter();
    for (tap_id, mut order_start_timer) in taps.iter_mut() {
        order_start_timer.0.tick(time.delta());
        if !order_start_timer.0.finished() {
            continue;
        } 
        let pending_order = match pending_orders.next() {
            Some((entity, order)) => {
                commands.entity(entity).despawn();
                order.clone()
            }
            None => break,
        };

        commands
            .entity(tap_id)
            .remove::<OpenForOrder>()
            .with_children(|parent| {
                parent
                    .spawn((
                        Cup,
                        pending_order.0.clone(),
                        Transform::from_xyz(0., -TAP_HEIGHT / 2. + CUP_HEIGHT / 2., 40.),
                        Visibility::Visible,
                    ))
                    .with_children(|cup| {
                        //spawn walls
                        cup.spawn((
                            CupWall,
                            Mesh2d(mesh_handles.cup_wall_mesh.clone()),
                            MeshMaterial2d(mesh_handles.cup_material.clone()),
                            RigidBody::Fixed,
                            Transform::from_xyz(-CUP_WIDTH / 2. + CUP_THICKNESS / 2., 0., 0.),
                        ));
                        cup.spawn((
                            CupWall,
                            Mesh2d(mesh_handles.cup_wall_mesh.clone()),
                            MeshMaterial2d(mesh_handles.cup_material.clone()),
                            RigidBody::Fixed,
                            Transform::from_xyz(CUP_WIDTH / 2. - CUP_THICKNESS / 2., 0., 0.),
                        ));
                        cup.spawn((
                            CupWall,
                            Mesh2d(mesh_handles.cup_base_mesh.clone()),
                            MeshMaterial2d(mesh_handles.cup_material.clone()),
                            RigidBody::Fixed,
                            Transform::from_xyz(0., -CUP_HEIGHT / 2. + CUP_THICKNESS / 2., 0.),
                        ));

                        //put dividers between different colors
                        let mut dividers: Vec<usize> = Vec::new();
                        let sections = &pending_order.0.sections;

                        for i in 1..sections.len() {
                            if sections[i] != sections[i - 1] {
                                dividers.push(i - 1);
                            }
                        }

                        for divider_pos in dividers.iter() {
                            cup.spawn((
                                CupDivider,
                                Mesh2d(mesh_handles.divider_mesh.clone()),
                                MeshMaterial2d(mesh_handles.divider_material.clone()),
                                Transform::from_xyz(
                                    0.0,
                                    (CUP_HEIGHT / -2.)
                                        + CUP_THICKNESS
                                        + (CUP_HEIGHT / sections.len() as f32
                                            * *divider_pos as f32),
                                    0.0,
                                ),
                            ));
                        }

                        //spawn bottom collider that collects drops
                        cup.spawn((
                            CupFillCollider,
                            Collider::cuboid(CUP_WIDTH / 2. - CUP_THICKNESS * 2., CUP_THICKNESS / 2.),
                            Transform::from_xyz(0.0, (CUP_HEIGHT / -2.) + CUP_THICKNESS, 0.0),
                            Sensor,
                            ActiveEvents::COLLISION_EVENTS,
                        ));
                    });
            });
    }
}

#[derive(Component)]
struct CupFill;

fn add_drops_to_cups(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut active_orders: Query<(Entity, &mut Order, &Parent)>,
    mut colliders: Query<(&mut Transform, &Parent), With<CupFillCollider>>,
    drops: Query<(&ColorDrop, Entity)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for event in collision_events.read() {
        match event {
            CollisionEvent::Started(entity, entity1, collision_event_flags) => {
                if *collision_event_flags != CollisionEventFlags::SENSOR {
                    continue;
                }

                let (mut transform, collider_parent) = match colliders.get_mut(*entity) {
                    Ok(res) => res,
                    Err(_) => match colliders.get_mut(*entity1) {
                        Ok(res) => res,
                        Err(_) => continue,
                    },
                };
                let (color, drop_entity) = match drops.get(*entity).or_else(|_| drops.get(*entity1))
                {
                    Ok(res) => res,
                    Err(_) => continue,
                };

                let (order_entity, mut order, tap_id) = match active_orders.get_mut(collider_parent.get()) {
                    Ok(res) => res,
                    Err(_) => continue,
                };

                if order.recieved.len() >= order.sections.len() {
                    if let Some(entity) = commands.get_entity(order_entity) {
                        entity.despawn_recursive();
                        commands.entity(drop_entity).despawn();
                        commands.entity(tap_id.get()).insert(OpenForOrder(Timer::new(Duration::from_secs(2), bevy::time::TimerMode::Once)));
                    }
                    continue;
                }
                order.recieved.push(color.0.clone());

                let section_height = (CUP_HEIGHT - CUP_THICKNESS) / order.sections.len() as f32;
                commands.entity(order_entity).with_child((
                    CupFill,
                    Mesh2d(meshes.add(Rectangle::new(
                        CUP_WIDTH - (CUP_THICKNESS * 2.),
                        section_height,
                    ))),
                    MeshMaterial2d(materials.add(color.0.clone())),
                    Transform::from_xyz(
                        0.0,
                        (CUP_HEIGHT / -2.)
                            + (CUP_THICKNESS / 2.)
                            + section_height * order.recieved.len() as f32
                            - 1.,
                        -1.0,
                    ),
                ));

                commands.entity(drop_entity).despawn();

                transform.translation.y += section_height;
            }
            _ => {}
        }
    }
}
