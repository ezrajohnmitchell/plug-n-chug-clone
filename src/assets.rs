use bevy::{app::Plugin, asset::Handle, ecs::system::Resource, image::Image, text::Font};
use bevy_asset_loader::asset_collection::AssetCollection;
use toml_loader::{TomlAsset, TomlAssetPlugin};

pub mod toml_loader;

pub struct AssetInitializerPlugin;

impl Plugin for AssetInitializerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(TomlAssetPlugin);
    }
}

#[derive(AssetCollection, Resource)]
pub struct OrderAssets {
    #[asset(path = "orders.toml")]
    pub order_types: Handle<TomlAsset>,
    #[asset(path = "cup_config.toml")]
    pub cup_config: Handle<TomlAsset>,
    #[asset(path = "fonts/ARCADECLASSIC.TTF")]
    pub order_font: Handle<Font>,
    #[asset(path = "sprites/cup-small.png")]
    pub cup_small: Handle<Image>,
    #[asset(path = "sprites/cup-medium.png")]
    pub cup_medium: Handle<Image>,
    #[asset(path = "sprites/cup-large.png")]
    pub cup_large: Handle<Image>,
    #[asset(path = "sprites/cup-handle.png")]
    pub cup_handle: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct BarAssets {
    #[asset(path = "sprites/bar-table.png")]
    pub bar_table: Handle<Image>,
    #[asset(path = "sprites/background.png")]
    pub background: Handle<Image>,
    #[asset(path = "sprites/taps.png")]
    pub taps: Handle<Image>,
}
