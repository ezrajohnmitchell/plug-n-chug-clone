use bevy::{prelude::*, render::render_resource::{AsBindGroup, ShaderRef}, sprite::Material2d};


#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct StatusBarMaterial {
    #[uniform(0)] pub foreground_color: LinearRgba,
    #[uniform(0)] pub background_color: LinearRgba,
    #[uniform(0)] pub percent: f32
}

impl StatusBarMaterial {
    pub fn new() -> Self {
        Self{
            foreground_color: LinearRgba::GREEN,
            background_color: LinearRgba::RED,
            percent: 1.0,
        }
    }
}

impl Material2d for StatusBarMaterial {
    fn fragment_shader() -> ShaderRef {
        "status-bar.wgsl".into() 
    }
}