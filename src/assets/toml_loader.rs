use bevy::{
    app::Plugin,
    asset::{Asset, AssetApp, AssetLoader},
    reflect::TypePath,
};
use thiserror::Error;

pub struct TomlAssetPlugin;

impl Plugin for TomlAssetPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.register_asset_loader(TomlLoader)
            .init_asset::<TomlAsset>();
    }
}

#[derive(Default)]
pub struct TomlLoader;

#[derive(Debug, TypePath, Asset)]
pub struct TomlAsset(pub String);

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TomlLoaderError {
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
    #[error("file is not utf-8")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("toml format incorrect")]
    Toml,
}

impl AssetLoader for TomlLoader {
    type Asset = TomlAsset;

    type Settings = ();

    type Error = TomlLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let toml_str = std::str::from_utf8(&bytes)?;
        let asset = TomlAsset(toml_str.to_owned());

        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}
