//! This module contains two types: `LayerStorage` and `TileStorage`.

use std::error::Error;
use std::fmt::Display;

use bevy::asset::HandleId;
use bevy::prelude::*;
use bevy::utils::HashMap;

// ───── Current Crate Imports ────────────────────────────────────────────── //

use super::tile_pos::TilePos;

// ───── Body ─────────────────────────────────────────────────────────────── //

type LayerIdx = usize;
type TilemapSize = UVec2;

#[derive(Component, Default, Debug)]
pub struct LayerStorage {
    pub layers: HashMap<Name, Entity>,
    pub asset_id: Option<HandleId>,
}

/// Errors which can be returned when working with `TileStorage` type.
#[derive(Debug, PartialEq, Eq)]
pub enum TileStorageError {
    /// Layer with this index was already initialized.
    LayerAlreadyInitialized,
    /// There are no layer with this index.
    NoLayerWithIndex,
    /// The tile's reserved cell is existing, but empty.
    TileCellEmpty,
    /// The tile position lies within the underlying tile layer's extents.
    TileOutOfLayer,
}

impl Display for TileStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TileStorageError::LayerAlreadyInitialized => {
                f.write_str("Layer already initialized!")?
            }
            TileStorageError::NoLayerWithIndex => {
                f.write_str("No layer with that index!")?
            }
            TileStorageError::TileCellEmpty => {
                f.write_str("Tile cell is empty!")?
            }
            TileStorageError::TileOutOfLayer => {
                f.write_str("That tile is out of layer bounds!")?
            }
        }
        Ok(())
    }
}

impl Error for TileStorageError {}

/// Stores all tiles entities of all layers of the map.
#[derive(Component, Default, Debug)]
pub struct TileStorage {
    tiles: HashMap<LayerIdx, (TilemapSize, Vec<Option<Entity>>)>,
}

impl TileStorage {
    /// Create new `TileStorage`
    pub fn new() -> Self {
        TileStorage {
            tiles: HashMap::new(),
        }
    }

    /// Reserves place for tiles from one layer
    pub fn init_place(
        &mut self,
        layer_idx: usize,
        size: UVec2,
    ) -> Result<(), TileStorageError> {
        if self.tiles.contains_key(&layer_idx) {
            return Err(TileStorageError::LayerAlreadyInitialized);
        }
        let vec = vec![None; size.x as usize * size.y as usize];
        self.tiles.insert(layer_idx, (size, vec));
        Ok(())
    }

    /// Gets a tile entity for the given tile position, if an entity is associated with that tile
    pub fn get(
        &self,
        layer_idx: usize,
        tile_pos: &TilePos,
    ) -> Result<Entity, TileStorageError> {
        if let Some((size, vec)) = self.tiles.get(&layer_idx) {
            if tile_pos.within_map_bounds(*size) {
                if let Some(entity) = vec[tile_pos.to_index(*size)] {
                    Ok(entity)
                } else {
                    Err(TileStorageError::TileCellEmpty)
                }
            } else {
                Err(TileStorageError::TileOutOfLayer)
            }
        } else {
            Err(TileStorageError::NoLayerWithIndex)
        }
    }

    /// Sets a tile entity for the given tile position, if the tile position lies within the
    /// underlying tile map's extents.
    ///
    /// If there is an entity already at that position, it will be replaced.
    pub fn set(
        &mut self,
        layer_idx: usize,
        tile_pos: &TilePos,
        tile_entity: Entity,
    ) -> Result<(), TileStorageError> {
        if let Some((size, ref mut vec)) = self.tiles.get_mut(&layer_idx) {
            if tile_pos.within_map_bounds(*size) {
                vec[tile_pos.to_index(*size)].replace(tile_entity);
                Ok(())
            } else {
                Err(TileStorageError::TileOutOfLayer)
            }
        } else {
            Err(TileStorageError::NoLayerWithIndex)
        }
    }

    /// Returns an iterator with all of the entities of the layer.
    pub fn iter_layer<'a>(
        &'a self,
        layer_idx: usize,
    ) -> Box<dyn Iterator<Item = &Option<Entity>> + 'a> {
        if let Some((_, vec)) = self.tiles.get(&layer_idx) {
            Box::new(vec.iter())
        } else {
            Box::new(std::iter::empty())
        }
    }

    /// Returns an mutable iterator with all of the entities of the layer.
    pub fn iter_mut_layer<'a>(
        &'a mut self,
        layer_idx: usize,
    ) -> Box<dyn Iterator<Item = &mut Option<Entity>> + 'a> {
        if let Some((_, vec)) = self.tiles.get_mut(&layer_idx) {
            Box::new(vec.iter_mut())
        } else {
            Box::new(std::iter::empty())
        }
    }

    /// Returns an iterator with all of the entities of the map, not ordered.
    pub fn iter_all(&self) -> impl Iterator<Item = &Option<Entity>> {
        self.tiles.values().map(|(_, vec)| vec).flatten()
    }

    /// Returns mutable iterator with all of the positions in the grid.
    pub fn iter_mut_all(
        &mut self,
    ) -> impl Iterator<Item = &mut Option<Entity>> {
        self.tiles.values_mut().map(|(_, vec)| vec).flatten()
    }

    /// Remove any stored entity at the given tile position, if the given `tile_pos` does lie within
    /// the extents of the underlying map.
    pub fn remove_at_layer(
        &mut self,
        layer_idx: usize,
        tile_pos: &TilePos,
    ) -> Result<Option<Entity>, TileStorageError> {
        if let Some((size, vec)) = self.tiles.get_mut(&layer_idx) {
            if tile_pos.within_map_bounds(*size) {
                Ok(vec[tile_pos.to_index(*size)].take())
            } else {
                Err(TileStorageError::TileOutOfLayer)
            }
        } else {
            Err(TileStorageError::NoLayerWithIndex)
        }
    }

    /// Clear all entities from storage.
    pub fn clear(&mut self) {
        self.tiles.clear();
    }
}

mod tests {
    use super::*;

    fn initialize_tile_storage() -> TileStorage {
        let mut tiles = HashMap::new();
        for i in 1..=3 {
            tiles.insert(
                i as usize,
                (
                    UVec2::new(2, 2),
                    vec![
                        Some(Entity::from_raw(i)),
                        Some(Entity::from_raw(i + 1)),
                        None,
                        Some(Entity::from_raw(i + 2)),
                    ],
                ),
            );
        }
        TileStorage { tiles }
    }

    #[test]
    fn test_init_place() {
        let mut tile_storage = TileStorage::new();
        let _ = tile_storage.init_place(1, UVec2::new(10, 10));
        assert_eq!(tile_storage.tiles.len(), 1);
        assert_eq!(tile_storage.tiles.get(&1).unwrap().0, UVec2::new(10, 10));
        assert_eq!((tile_storage.tiles.get(&1).unwrap().1).len(), 100);
    }

    #[test]
    fn test_set_get() {
        let mut tile_storage = TileStorage::new();
        let _ = tile_storage.init_place(1, UVec2::new(10, 10));
        // Should be error
        assert_eq!(
            tile_storage.get(1, &TilePos::new(1, 1)),
            Err(TileStorageError::TileCellEmpty)
        );

        // Should be error
        assert_eq!(
            tile_storage.set(2, &TilePos::new(1, 1), Entity::from_raw(1)),
            Err(TileStorageError::NoLayerWithIndex)
        );

        // Should be Ok
        assert_eq!(
            tile_storage.set(1, &TilePos::new(0, 0), Entity::from_raw(1)),
            Ok(())
        );
        assert_eq!(
            tile_storage.tiles.get(&1).unwrap().1[0],
            Some(Entity::from_raw(1))
        );

        // Should be Ok
        assert_eq!(
            tile_storage.set(1, &TilePos::new(0, 1), Entity::from_raw(2)),
            Ok(())
        );
        assert_eq!(
            tile_storage.tiles.get(&1).unwrap().1[10],
            Some(Entity::from_raw(2))
        );
    }

    #[test]
    fn test_iter_layer() {
        let tile_storage = initialize_tile_storage();
        let mut iter = tile_storage.iter_layer(1);
        assert_eq!(iter.next(), Some(&Some(Entity::from_raw(1))));
        assert_eq!(iter.next(), Some(&Some(Entity::from_raw(2))));
        assert_eq!(iter.next(), Some(&None));
        assert_eq!(iter.next(), Some(&Some(Entity::from_raw(3))));
        assert_eq!(tile_storage.iter_layer(1).count(), 4);
    }

    #[test]
    fn test_iter_all() {
        let tile_storage = initialize_tile_storage();
        let iter = tile_storage.iter_all();
        assert_eq!(iter.count(), 12);
    }

    #[test]
    fn test_remove_at_layer() {
        let mut tile_storage = initialize_tile_storage();
        assert_eq!(tile_storage.iter_all().count(), 12);
        let removed = tile_storage.remove_at_layer(3, &TilePos::new(1, 1));
        assert_eq!(removed, Ok(Some(Entity::from_raw(5))));

        let removed = tile_storage.remove_at_layer(1, &TilePos::new(0, 1));
        assert_eq!(removed, Ok(None));

        assert_eq!(tile_storage.iter_all().count(), 12);
    }
}
