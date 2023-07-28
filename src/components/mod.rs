//! Components in this module:
//! `LayerStorage` - stores all layers of the map.
//! `TileStorage` - stores all tiles of the map.
//! `TilePos` - represents integer tile position in the tilemap.
//! `TilesetTexture` - stores image handles for the tilesets.
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
pub enum TilesetTexture {
    /// All textures for tiles are inside a single image asset.
    Single(Handle<Image>),
    /// One texture for each tile.
    Vector(Vec<Handle<Image>>),
}

impl Default for TilesetTexture {
    fn default() -> Self {
        TilesetTexture::Single(Default::default())
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
    // properties: HashMap<String, String>,
}
