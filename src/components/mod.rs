//! [Component]'s to spawning with tiles or tilemap.

use bevy::prelude::*;
use bevy::utils::HashMap;

// ───── Current Crate Imports ────────────────────────────────────────────── //

pub use storages::{LayerStorage, TileStorage, TileStorageError};
pub use tile_pos::TilePos;

// ───── Submodules ───────────────────────────────────────────────────────── //

mod storages;
mod tile_pos;

// ───── Body ─────────────────────────────────────────────────────────────── //

/// Insert this component to `Handle<TiledMapAsset>` entity to despawn tilemap.
/// ```
/// use bevy::prelude::*;
/// use bevy_tiled_toolkit::prelude::*;
///
/// fn system_despawn_map(
///     mut commands: Commands,
///     input: Res<Input<KeyCode>>,
///     tilemap_query: Query<Entity, With<Handle<TiledMapAsset>>>,
/// ) {
///     if input.just_pressed(KeyCode::Space) {
///         for entity in tilemap_query.iter() {
///             commands.entity(entity).insert(DespawnTiledMap);
///         }
///     }
/// }
/// ```
#[derive(Component)]
pub struct DespawnTiledMap;

/// Represents unified container for tilesets from single spritesheets and
/// inidividual images.
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

/// Missing documentation
#[derive(Component)]
pub struct Animation {
    /// Missing documentation
    pub frames: Vec<tiled::Frame>,
    /// Missing documentation
    pub current_frame: tiled::TileId,
    /// Missing documentation
    pub offsets: HashMap<tiled::TileId, usize>,
    /// Missing documentation
    pub timer: Timer,
}

#[derive(Component)]
pub(crate) struct NeedToSpawn;
