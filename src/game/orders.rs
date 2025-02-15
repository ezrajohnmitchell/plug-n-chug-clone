use std::{any::Any, char::MAX, time::Duration};

use bevy::{
    app::{Plugin, Update}, asset::{Assets, Handle}, color::Color, ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        query::With,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut, Resource},
    }, hierarchy::{BuildChildren, ChildBuild, Children, DespawnRecursiveExt, Parent}, math::primitives::Rectangle, render::{
        mesh::{Mesh, Mesh2d},
        view::Visibility,
    }, sprite::{ColorMaterial, Material2d, MeshMaterial2d, Sprite}, state::{
        condition::in_state,
        state::{NextState, OnEnter, OnExit, State},
    }, text::{Text2d, TextFont, TextLayout}, time::{Time, Timer}, transform::components::Transform, utils::{
        default,
        hashbrown::{
            hash_map::Entry::{Occupied, Vacant},
            HashMap,
        },
    }
};
use bevy_rapier2d::{
    prelude::{ActiveEvents, Collider, CollisionEvent, Sensor},
    rapier::prelude::CollisionEventFlags,
};
use order_config::{CupConfig, OrderConfig, OrderList, SectionConfig};
use rand::{distr::{Distribution, StandardUniform}, seq::{IndexedRandom, IteratorRandom}, Rng};

use crate::{
    assets::{toml_loader::TomlAsset, OrderAssets},
    GameStates,
};

use super::{
    status_bar::StatusBarMaterial, taps::{ColorDrop, Tap}, Event::FailedOrder, GameScreen, LevelState, StatePlugin
};

mod order_config;

pub struct OrderPlugin(GameStates);

impl Plugin for OrderPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(OnEnter(self.0.clone()), (setup_orders, setup_cup_meshes));
        app.add_systems(
            Update,
            (
                spawn_orders,
                assign_pending_orders,
                add_drops_to_cups,
                add_next_order_type,
                update_order_timers,
            )
                .run_if(in_state(self.0.clone())),
        );
        app.add_systems(OnExit(self.0.clone()), remove_resources);
    }
}

impl StatePlugin<OrderPlugin> for OrderPlugin {
    fn run_on_state(state: GameStates) -> OrderPlugin {
        OrderPlugin(state)
    }
}

fn remove_resources(mut commands: Commands) {
    commands.remove_resource::<AvailableOrders>();
    commands.remove_resource::<CupMaterials>();
    commands.remove_resource::<OrderSpawnTimer>();
    commands.remove_resource::<OrdersWithDifficulty>();
    commands.remove_resource::<CupConfig>();
}

#[derive(Component, Debug, Clone)]
pub struct PendingOrder(Order);

#[derive(Component, Debug, Clone)]
pub struct Order{
    order_type: OrderType,
    recieved: Vec<Color>,
    time_remaining: Timer,
    size: OrderSize
}

#[derive(Component, Debug, Clone)]
pub struct OrderType {
    sections: Vec<Section>, //treat 0 as buttom of the cup
    name: String,
}

impl From<&OrderConfig> for OrderType {
    fn from(value: &OrderConfig) -> Self {
        let sections: Vec<Section> = value.sections.iter().map(|section_config| Section::from(section_config)).collect();

        Self {
            sections,
            name: value.name.clone(),
        }

    }
}

#[derive(Debug, Clone)]
pub struct Section{
    pub color: Color,
    pub size: usize
}

impl From<&SectionConfig> for Section {
    fn from(value: &SectionConfig) -> Self {
            let color = Color::linear_rgb(
                value.color[0],
                value.color[1],
                value.color[2],
            );

            Self {
                color,
                size: value.size
            }
    }
}

#[derive(Debug, Clone)]
enum OrderSize {
    Small,
    Medium,
    Large
}

impl Default for OrderSize {
    fn default() -> Self {
        OrderSize::Medium
    }
}

impl Distribution<OrderSize> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> OrderSize {
        match rng.random_range(0..3) {
           0 => OrderSize::Small,
           1 => OrderSize::Medium,
           _ => OrderSize::Large
        }
    }
}

#[derive(Resource)]
pub struct AvailableOrders(Vec<OrderType>);

#[derive(Resource)]
pub struct OrdersWithDifficulty(HashMap<u32, Vec<OrderType>>);

#[derive(Component)]
struct OrderDifficutlyAdder(Timer);

impl OrdersWithDifficulty {
    fn get_starter_orders(&mut self) -> Vec<OrderType> {
        self.0
            .remove(&0)
            .expect("no orders were defined at difficulty 0")
    }

    fn get_lowest_difficulty_order(&mut self) -> Option<OrderType> {
        let order_option = match self.0.keys().min() {
            Some(key) => match self.0.entry(*key) {
                Occupied(o) => o.into_mut().pop(),
                Vacant(_) => None,
            },
            None => None,
        };
        self.0.retain(|_, value| value.len() > 0);

        order_option
    }
}

pub fn setup_orders(
    mut commands: Commands,
    order_asset: Res<OrderAssets>,
    toml_assets: Res<Assets<TomlAsset>>,
) {
    let toml_str = toml_assets
        .get(order_asset.order_types.id())
        .expect("orders.toml is missing")
        .0
        .as_str();
    let order_list: OrderList = toml::from_str(toml_str).expect("orders.toml format is incorrect");

    let mut orders: HashMap<u32, Vec<OrderType>> = HashMap::new();
    order_list.orders.iter().for_each(|order_config| {
        let order_type = OrderType::from(order_config);

        match orders.entry(order_config.difficulty) {
            Occupied(o) => {
                let orders = o.into_mut();
                orders.push(order_type);
            }
            Vacant(v) => {
                v.insert(vec![order_type]);
            }
        };
    });

    let mut orders = OrdersWithDifficulty(orders);

    commands.insert_resource(AvailableOrders(orders.get_starter_orders()));
    commands.insert_resource(orders);
    commands.insert_resource(OrderSpawnTimer(Timer::new(
        Duration::from_secs(3),
        bevy::time::TimerMode::Repeating,
    )));
    commands.spawn((
        OrderDifficutlyAdder(Timer::new(
            Duration::from_secs(5),
            bevy::time::TimerMode::Repeating,
        )),
        GameScreen,
    ));
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
        if let Some(order_type) = order_to_spawn {
            commands.spawn(PendingOrder(Order {
                order_type: order_type.clone(),
                recieved: Vec::new(),
                time_remaining: Timer::new(Duration::from_secs(60), bevy::time::TimerMode::Once),
                size: rand::rng().random()
            }));
        }
    }
}

fn add_next_order_type(
    mut available_order: ResMut<AvailableOrders>,
    mut orders_with_difficutly: ResMut<OrdersWithDifficulty>,
    mut query: Query<&mut OrderDifficutlyAdder>,
    time: Res<Time>,
) {
    if let Ok(mut timer) = query.get_single_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            if let Some(order) = orders_with_difficutly.get_lowest_difficulty_order() {
                available_order.0.push(order);
            }
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
struct CupMaterials {
    divider_material: Handle<ColorMaterial>,
}

pub fn setup_cup_meshes(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    toml_assets: Res<Assets<TomlAsset>>,
    order_asset: Res<OrderAssets>,
) {
    let toml_str = toml_assets
        .get(order_asset.cup_config.id())
        .expect("orders.toml is missing")
        .0
        .as_str();
    let cup_config: CupConfig = toml::from_str(toml_str).expect("orders.toml format is incorrect");

    commands.insert_resource(CupMaterials {
        divider_material: materials.add(Color::linear_rgb(cup_config.divider_color[0], cup_config.divider_color[1], cup_config.divider_color[2])),
    });
    commands.insert_resource(cup_config);
}

#[derive(Component)]
struct CupFillCollider;

#[derive(Component)]
pub struct OpenForOrder(Timer);
impl OpenForOrder {
    pub fn new() -> OpenForOrder {
        OpenForOrder(Timer::new(
            Duration::from_secs(2),
            bevy::time::TimerMode::Once,
        ))
    }
}

fn assign_pending_orders(
    mut commands: Commands,
    pending_orders: Query<(Entity, &PendingOrder)>,
    mut taps: Query<(Entity, &mut OpenForOrder), With<Tap>>,
    divider_material: Res<CupMaterials>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
    order_assets: Res<OrderAssets>,
    cup_config: Res<CupConfig>,
    mut status_bar_material: ResMut<Assets<StatusBarMaterial>>
) {
    let mut pending_orders = pending_orders
        .iter()
        .choose_multiple(&mut rand::rng(), pending_orders.iter().len())
        .into_iter();
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
                        Sprite::from_image(match &pending_order.0.size {
                            OrderSize::Small => order_assets.cup_small.clone(),
                            OrderSize::Medium => order_assets.cup_medium.clone(),
                            OrderSize::Large => order_assets.cup_large.clone(),
                        }),
                        pending_order.0.clone(),
                        Transform::from_xyz(0., -12. + (- 151. / 2.) + cup_config.cup_height / 2., 40.),
                        Visibility::Visible,
                    ))
                    .with_children(|cup| {
                        let (cup_width, cup_inner_width)= match &pending_order.0.size {
                                OrderSize::Small => (cup_config.cup_small_width, cup_config.cup_small_inner_width),
                                OrderSize::Medium => (cup_config.cup_medium_width, cup_config.cup_medium_inner_width),
                                OrderSize::Large => (cup_config.cup_large_width, cup_config.cup_large_inner_width),
                            };

                        //spawn handle
                        cup.spawn((
                            Sprite::from_image(order_assets.cup_handle.clone()),
                            Transform::from_xyz(cup_width / 2. + 2., 0., 0.)
                        ));

                        //put dividers between different colors
                        let mut dividers: Vec<usize> = Vec::new();
                        let order_size_multiplier = get_order_size(&pending_order.0.size);
                        let total_sections = pending_order.0.order_type.sections.iter().fold(0 as usize, |acc, section| {
                            let val = acc + (section.size * order_size_multiplier);
                            dividers.push(val);
                            val
                        });
                        dividers.pop();

                        for divider_pos in dividers.iter() {
                            cup.spawn((
                                CupDivider,
                                Mesh2d(meshes.add(Rectangle::new(cup_inner_width, 2.))),
                                MeshMaterial2d(divider_material.divider_material.clone()),
                                Transform::from_xyz(
                                    0.0,
                                    (cup_config.cup_height / -2. + cup_config.cup_bottom_thickness)
                                        + (cup_config.cup_height / total_sections as f32
                                            * *divider_pos as f32),
                                    0.0,
                                ),
                            ));
                        }

                        //spawn bottom collider that collects drops
                        cup.spawn((
                            CupFillCollider,
                            Collider::cuboid(
                                cup_inner_width / 2.,
                                (cup_config.cup_height - cup_config.cup_bottom_thickness)
                                    / total_sections as f32
                                    / 2. + cup_config.cup_bottom_thickness,
                            ),
                            Transform::from_xyz(0.0, cup_config.cup_height / -2., 0.0),
                            Sensor,
                            ActiveEvents::COLLISION_EVENTS,
                        ));

                        //order name
                        cup.spawn((
                            Text2d::new(pending_order.0.order_type.name.clone()),
                            TextFont {
                                font: order_assets.order_font.clone(),
                                font_size: 20.,
                                ..default()
                            },
                            TextLayout::new_with_justify(bevy::text::JustifyText::Center),
                            Transform::from_xyz(0., -cup_config.cup_height / 2. - 25., 5.),
                        ));

                        let status_bar_material = status_bar_material.add(StatusBarMaterial::new());

                        //order status bar 
                        cup.spawn((
                            CupStatusBar(status_bar_material.clone()),
                            Mesh2d(meshes.add(Rectangle::new(cup_config.status_bar_width, 20.))),
                            MeshMaterial2d(status_bar_material),
                            Transform::from_xyz(1., -cup_config.cup_height / 2. - 50., 0.)
                        ));
                    });
            });
    }
}

#[derive(Component)]
struct CupStatusBar(Handle<StatusBarMaterial>);

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
    cup_config: Res<CupConfig>,
    state: Res<State<LevelState>>,
    mut next_state: ResMut<NextState<LevelState>>,
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

                let (order_entity, mut order, tap_id) =
                    match active_orders.get_mut(collider_parent.get()) {
                        Ok(res) => res,
                        Err(_) => continue,
                    };
                let total_sections: usize = order.order_type.sections.iter().map(|section| section.size).sum::<usize>() * get_order_size(&order.size);

                if order.recieved.len() >= total_sections {
                    if let Some(entity) = commands.get_entity(order_entity) {
                        entity.despawn_recursive();
                        commands.entity(drop_entity).despawn();
                        commands
                            .entity(tap_id.get())
                            .insert(OpenForOrder(Timer::new(
                                Duration::from_secs(2),
                                bevy::time::TimerMode::Once,
                            )));
                            if is_cup_failed(&order.order_type.sections, &order.recieved, &order.size) {
                                next_state.set(state.get().next(&FailedOrder));
                            }
                    }
                    continue;
                }
                order.recieved.push(color.0.clone());

                let cup_inner_width = match &order.size {
                    OrderSize::Small => cup_config.cup_small_inner_width,
                    OrderSize::Medium => cup_config.cup_medium_inner_width,
                    OrderSize::Large => cup_config.cup_large_inner_width,
                };

                let section_height = (cup_config.cup_height - cup_config.cup_bottom_thickness)/ total_sections as f32;
                commands.entity(order_entity).with_child((
                    CupFill,
                    Mesh2d(meshes.add(Rectangle::new(
                        cup_inner_width,
                        section_height,
                    ))),
                    MeshMaterial2d(materials.add(color.0.clone())),
                    Transform::from_xyz(
                        0.0,
                        ((cup_config.cup_height  - cup_config.cup_bottom_thickness)/ -2.)
                            + section_height * order.recieved.len() as f32
                            + cup_config.cup_bottom_thickness,
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

const COLOR_RANGE: f32 = 20.;
const MAX_FAILURES_PER_SECTION: usize = 4;

fn is_cup_failed(expected: &Vec<Section>, recieved: &Vec<Color>, size: &OrderSize) -> bool {
    let mut index = 0;
    for section in expected.iter() {
        let section_size = section.size * get_order_size(size);

        if index + section_size > recieved.len() {
            return true;
        };
        let slice = &recieved[index..section_size + index];
        let equal_colors = slice
            .iter()
            .filter(|recieved_color| color_equal(&section.color, *recieved_color, COLOR_RANGE))
            .count();
        if section_size - equal_colors > MAX_FAILURES_PER_SECTION {
            return true;
        }
        index += section_size;
    }

    false
}

fn color_equal(color1: &Color, color2: &Color, range: f32) -> bool {
    let color1 = color1.to_linear();
    let color2 = color2.to_linear();

    let red = color1.red;
    let blue = color2.blue;
    let green = color2.green;

    color2.red >= red - range
        && color2.red <= red + range
        && color2.blue >= blue - range
        && color2.blue <= blue + range
        && color2.green >= green - range
        && color2.green <= green + range
}

fn get_order_size(size: &OrderSize) -> usize {
        match size {
            OrderSize::Small => 1,
            OrderSize::Medium => 2,
            OrderSize::Large => 4,
        }
}

fn update_order_timers(
    mut orders: Query<(&mut Order, Entity, &Parent)>,
    status_bars: Query<(&CupStatusBar, &Parent)>,
    mut status_bar_materials: ResMut<Assets<StatusBarMaterial>>,
    time: Res<Time>,
    state: Res<State<LevelState>>,
    mut next_state: ResMut<NextState<LevelState>>,
    mut commands: Commands
){
    orders.iter_mut().for_each(|(mut order, _, _)| {
        order.time_remaining.tick(time.delta());
    });

    for (status_bar_handle, parent) in status_bars.iter() {
        if let Ok ((order, entity , tap)) = orders.get(parent.get()) {
            match status_bar_materials.get_mut(status_bar_handle.0.id()) {
                Some(material) => {
                    let timer = &order.time_remaining;
                    if timer.finished() {
                        //send failure event
                        next_state.set(state.get().next(&FailedOrder));
                        commands.entity(entity).despawn_recursive();
                        commands.entity(tap.get()).insert(OpenForOrder::new());
                        continue;
                    }

                    material.percent = 1.0 - timer.elapsed().div_duration_f32(timer.duration());
                },
                None => todo!(),
            }
        }
    }


}