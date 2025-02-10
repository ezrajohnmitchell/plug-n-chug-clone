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

#[derive(Deserialize)]
pub struct SectionConfig {
    pub color: [f32; 3],
    pub size: usize,
}
