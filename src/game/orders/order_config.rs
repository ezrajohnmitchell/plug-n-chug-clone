use bevy::ecs::system::Resource;
use serde::Deserialize;


#[derive(Deserialize)]
pub struct OrderList {
    pub orders: Vec<OrderConfig>,
}

#[derive(Deserialize)]
pub struct OrderConfig {
    pub name: String,
    pub sections: Vec<SectionConfig>,
    pub difficulty: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SectionConfig{
    pub color: [f32; 3],
    pub size: usize,
}

#[derive(Deserialize, Resource)]
pub struct CupConfig {
    pub cup_small_width: f32,
    pub cup_small_inner_width: f32,
    pub cup_medium_width: f32,
    pub cup_medium_inner_width: f32,
    pub cup_large_width: f32,
    pub cup_large_inner_width: f32,
    pub cup_height: f32,
    pub cup_bottom_thickness: f32,
    pub handle_width: f32,
    pub divider_color: [f32; 3],
    pub status_bar_width: f32
}
