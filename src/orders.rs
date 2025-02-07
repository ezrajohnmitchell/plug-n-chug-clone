use bevy::{color::Color, ecs::component::Component};

#[derive(Component)]
struct Drink(Vec<DrinkSection>);

struct DrinkSection {
    percent_of_drink: f32,
    color: Color,
}


#[derive(Component, Debug, Clone)]
pub struct Order;