//! This is a [Tiled](https://www.mapeditor.org) integration plugin for
//! [Bevy](https://bevyengine.org) game engine.
//!
//! ### Getting starged
//! This plugin allows to use almost every features from `Tiled` map editor in
//! in your Bevy-game.
//!
//! Follow these steps to start:
//! 1. Add the [TiledToolkitPlugin](self::plugin::TiledToolkitPlugin) to the [App].
//! 2. Spawn a [TiledMapBundle](self::plugin::TiledMapBundle):
//! ```
//! use bevy::prelude::*;
//! use bevy_tiled_toolkit::prelude::*;
//!
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
//! If you want to spawn your custom component with particular tile, follow
//! these instructions:
//! 1. Declare your custom component:
//! ```
//! use bevy::prelude::*;
//!
//! #[derive(Component, Default)]
//! struct Ninja {
//!     strength: f32,
//! }
//! ```
//! 2. Implement [TiledComponent] trait for your type and register your type
//! in the [App]:
//! ```
//! use bevy::prelude::*;
//! use bevy::ecs::system::EntityCommands;
//! use bevy::utils::HashMap;
//! use bevy::log;
//! use bevy_tiled_toolkit::prelude::*;
//!
//! #[derive(Component, Default)]
//! struct Ninja {
//!     strength: f32,
//! }
//!
//! impl TiledComponent for Ninja {
//!     fn insert_self_to_entity(
//!         &self,
//!         commands: &mut EntityCommands,
//!         values: HashMap<String, tiled::PropertyValue>,
//!         asset_server: &Res<AssetServer>,
//!     ) {
//!         for (key, value) in values {
//!             if key == String::from("strength") {
//!                 let tiled::PropertyValue::FloatValue(v) = value else {
//!                     log::error!("Cant spawn Ninja, wrong PropertyValue type");
//!                     continue;
//!                 };
//!                 commands.insert(Ninja { strength: v });
//!             }
//!         }
//!     }
//!     fn get_class_name(&self) -> &str {
//!         "Ninja"
//!     }
//! }
//!
//! let mut app = App::new();
//! app.register_tiled_component::<Ninja>();
//! ```
//! 3. In `Tiled`, in the `Class` field of your tile, insert the same name
//! that `get_class_name` function returns.
//! 4. Create properties in `Tiled` for your tile, and they will be passed to
//! `insert_self_to_entity` function, where you can use them to initialize your
//! component.
//!
//! ### Rendering with bevy_ecs_tilemap
//!
//! This crate supports rendering layers with [bevy_ecs_tilemap](https://github.com/StarArawn/bevy_ecs_tilemap),
//! but there are some limitations:
//! * Only tile layers supported.
//! * Each tile should have the same size.
//! * One layer should use only one tilemap at the same time.
//! * You can't implement YSorting with tiles spawned with `bevy_ecs_tilemap`,
//! because it's impossible to change transform for each tile independently:
//! `bevy_ecs_tilemap` glue together all tiles into one big image.
//!
//! At the same time it is recommended to render at least the base layer with
//! [bevy_ecs_tilemap](https://github.com/StarArawn/bevy_ecs_tilemap)
//! (because base layer usually completely filled with tiles and don't need
//! YSorting).
//!
//! If you spawn each tile as just [TextureAtlas]'es on such layer, perfomance will
//! be poor on mobile devices and low-end computers, especially on medium-sized
//! and big-sized maps, (there will be tile-flicker when moving camera on
//! iphone 8, for example) if map is bigger then 50x50.
//! [TextureAtlas]'es based rendering works good for tile layers, where there
//! are not too many tiles, and, naturally, `Object layers` can be rendered
//! only with [TextureAtlas]'es.
//!
//! To enable `bevy_ecs_tilemap` rendering on the particular layer, you should
//! create boolean property named `bevy_ecs_tilemap` on the desired layer in
//! Tiled, and click on checkbox of this property to activate it.
//!
//! ### Despawning tilemap
//! Spawn a [DespawnTilemap](self::components::DespawnTiledMap) component to an
//! entity with `Handle<TiledMapAsset>` to despawn the tilemap:
//! ```
//! use bevy::prelude::*;
//! use bevy_tiled_toolkit::asset_loader::TiledMapAsset;
//! use bevy_tiled_toolkit::components::DespawnTiledMap;
//!
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
    pub use crate::TiledComponent;
}

/// Spawn your components with specific tiles or objects from Tiled.
///
/// Implement this trait for your component type and add your component's name
/// to `Class` field in tile's properties in Tiled.
///
/// ```
/// use bevy::prelude::*;
/// use bevy::ecs::system::EntityCommands;
/// use bevy::utils::HashMap;
/// use bevy::log;
/// use bevy_tiled_toolkit::TiledComponent;
///
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
///         asset_server: &Res<AssetServer>,
///     ) {
///         for (key, value) in values {
///             if key == String::from("strength") {
///                 let tiled::PropertyValue::FloatValue(v) = value else {
///                     log::error!("Cant spawn Ninja, wrong PropertyValue type");
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
/// use bevy::prelude::*;
/// use bevy_tiled_toolkit::asset_loader::TiledMapAsset;
///
/// #[derive(Component, Default)]
/// struct Ninja {
///     strength: f32,
/// }
///
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
