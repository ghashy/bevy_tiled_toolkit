//! This is a tiled integration plugin for Bevy game engine.

#![deny(
    // warnings,
    // missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    // missing_docs
)]

// ───── Submodules ───────────────────────────────────────────────────────── //

use bevy::{ecs::system::EntityCommands, utils::HashMap};

// Top-level modules
mod app_extension;
mod asset_loader;
mod components;
mod plugin;
mod resources;

// ───── Body ─────────────────────────────────────────────────────────────── //

/// Groups all used types.
pub mod prelude {
    pub use super::asset_loader::TilemapAsset;
    pub use super::components::{
        LayerStorage, TilePos, TileStorage, TileStorageError,
    };
    pub use super::resources::{TiledPoint, TiledPoints};
    pub use crate::app_extension::TiledComponentReg;
    pub use crate::plugin::TiledMapBundle;
    pub use crate::plugin::TiledToolkitPlugin;
}

/// Spawn your components with specific tiles or objects from Tiled for fast
/// querying!
///
/// Implement this trait for your component type and add your component's name
/// to `Class` field in tile's properties in Tiled.
///
/// ```
/// #[derive(Component, Default)]
/// struct Ninja {
///     strength: f32,
/// }
///
/// impl TiledComponent for Ninja {
///     fn insert_self_to_entity(
///         &self,
///         commands: &mut EntityCommands,
///         values: HashMap<String, tiled::PropertyValue>,
///     ) {
///         for (key, value) in values {
///             if key == String::from("strength") {
///                 let tiled::PropertyValue::FloatValue(v) = value else {
///                     error!("Cant spawn Ninja, wrong PropertyValue type");
///                     continue;
///                 };
///                 println!("Spawning ninja!");
///                 commands.insert(Ninja { strength: v });
///             }
///         }
///     }
///     fn get_class_name(&self) -> &str {
///         "Ninja"
///     }
/// }
/// ```
/// Then your can query for `TextureAtlasSprite` of this tile or object:
/// ```
/// fn color_ninja(
///     mut ninja_query: Query<&mut TextureAtlasSprite, With<Ninja>>,
///     time: Res<Time>,
/// ) {
///     for mut ninja in ninja_query.iter_mut() {
///         ninja.color.set_g((time.elapsed_seconds() * 8.).sin().abs());
///     }
/// }
/// ```
pub trait TiledComponent {
    /// This function called when we are spawning tile with same `Class` name
    /// as your component type, and it gives `HashMap` with properties from
    /// Tiled.
    fn insert_self_to_entity(
        &self,
        commands: &mut EntityCommands,
        values: HashMap<String, tiled::PropertyValue>,
    );
    /// This function required for getting name of your type and compare it
    /// with `Class` name from Tiled.
    fn get_class_name(&self) -> &str;
}
