use bevy::{app::Plugin, asset::Handle, ecs::system::Resource, text::Font};
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
    #[asset(path="fonts/ARCADECLASSIC.TTF")]
    pub order_font: Handle<Font>
}
