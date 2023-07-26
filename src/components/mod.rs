//! Components in this module:
//! `LayerStorage` - stores all layers of the map.
//! `TileStorage` - stores all tiles of the map.
//! `TilePos` - represents integer tile position in the tilemap.
//! `TilemapTexture` - stores image handles for the tilesets.
//! `Tile` - marker component.
//! `Object` - marker component

use bevy::prelude::*;
use bevy::utils::HashMap;

// ───── Current Crate Imports ────────────────────────────────────────────── //

pub use storages::{LayerStorage, TileStorage, TileStorageError};
pub use tile_pos::TilePos;

// ───── Submodules ───────────────────────────────────────────────────────── //

mod storages;
mod tile_pos;

// ───── Body ─────────────────────────────────────────────────────────────── //

#[derive(Component, Reflect, Clone, Debug, Hash, PartialEq, Eq)]
#[reflect(Component)]
pub enum TilemapTexture {
    /// All textures for tiles are inside a single image asset.
    Single(Handle<Image>),
    /// One texture for each tile.
    Vector(Vec<Handle<Image>>),
}

impl Default for TilemapTexture {
    fn default() -> Self {
        TilemapTexture::Single(Default::default())
    }
}

pub type Duration = u32;

#[derive(Component)]
pub struct Animation {
    pub frames: Vec<tiled::Frame>,
    pub current_frame: tiled::TileId,
    // TODO: doc this
    pub offsets: HashMap<tiled::TileId, usize>,
    pub timer: Timer,
}

/// Component for marking objects loaded from tiled.
#[derive(Component)]
pub struct Object {
    name: Option<Name>,
    properties: HashMap<String, String>,
}

/// Component for marking objects loaded from tiled.
#[derive(Component)]
pub struct Tile {
    pub properties: HashMap<String, PropertyValue>,
}

pub enum PropertyValue {
    /// A boolean value. Corresponds to the `bool` property type.
    BoolValue(bool),
    /// A floating point value. Corresponds to the `float` property type.
    FloatValue(f32),
    /// A signed integer value. Corresponds to the `int` property type.
    IntValue(i32),
    /// A color value. Corresponds to the `color` property type.
    ColorValue(Color),
    /// A string value. Corresponds to the `string` property type.
    StringValue(String),
    /// A filepath value. Corresponds to the `file` property type.
    /// Holds the path relative to the map or tileset.
    FileValue(String),
    /// An object ID value. Corresponds to the `object` property type.
    /// Holds the id of a referenced object, or 0 if unset.
    ObjectValue(u32),
}
