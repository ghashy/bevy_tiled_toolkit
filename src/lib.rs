//! This is a tiled integration plugin for Bevy game engine.
//! ### Spawning tilemap
//! To spawn tilemap just spawn a [TiledMapBundle](self::plugin::TiledMapBundle)
//! :
//! ```
//! fn system_spawn_map(
//!     mut commands: Commands,
//!     asset_server: Res<AssetServer>,
//!     input: Res<Input<KeyCode>>,
//! ) {
//!     if input.just_pressed(KeyCode::Space) {
//!         let tiled_map: Handle<TiledMapAsset> =
//!             asset_server.load("tiled/tilemaps/Map.tmx");
//!
//!         commands.spawn(TiledMapBundle {
//!             tiled_map,
//!             name: Name::from("TiledMap"),
//!             ..default()
//!         });
//!     }
//! }
//! ```
//!
//! ### Despawning tilemap
//! Spawn a [DespawnTilemap](self::components::DespawnTiledMap) component to an
//! entity with `Handle<TiledMapAsset>`to despawn the tilemap:
//! ```
//! fn system_despawn_map(
//!     mut commands: Commands,
//!     input: Res<Input<KeyCode>>,
//!     tiled_map_query: Query<Entity, With<Handle<TiledMapAsset>>>,
//! ) {
//!     if input.just_pressed(KeyCode::P) {
//!         for entity in tiled_map_query.iter() {
//!             commands.entity(entity).insert(DespawnTiledMap);
//!         }
//!     }
//! }
//! ```

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

use bevy::prelude::*;
use bevy::{ecs::system::EntityCommands, utils::HashMap};

// ───── Submodules ───────────────────────────────────────────────────────── //

// Top-level modules
mod app_extension;
pub mod asset_loader;
pub mod components;
mod plugin;
mod resources;

// ───── Body ─────────────────────────────────────────────────────────────── //

pub mod prelude {
    //! `use bevy_tiled_toolkit::prelude::*;` to import commonly used items.
    pub use super::asset_loader::TiledMapAsset;
    pub use super::components::{
        LayerStorage, TilePos, TileStorage, TileStorageError,
    };
    pub use super::resources::{TiledPoint, TiledPoints};
    pub use crate::app_extension::TiledComponentReg;
    pub use crate::components::DespawnTiledMap;
    pub use crate::plugin::TiledMapBundle;
    pub use crate::plugin::TiledToolkitPlugin;
}

/// Spawn your components with specific tiles or objects from Tiled.
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
    /// This method called when we are spawning tile with same `Class` name
    /// as your component's type, and it provides `HashMap` with properties
    /// from Tiled. We need `&self` here because we do not know a particular
    /// type for which we will call the method, so we can't call it as a static
    /// method.
    fn insert_self_to_entity(
        &self,
        commands: &mut EntityCommands,
        values: HashMap<String, tiled::PropertyValue>,
        asset_server: &Res<AssetServer>,
    );
    /// This function required for getting name of your type and compare it
    /// with `Class` name from Tiled.
    fn get_class_name(&self) -> &str;
}
