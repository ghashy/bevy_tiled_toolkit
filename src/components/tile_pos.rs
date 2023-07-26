//! This module contains `TilePos` type.

use bevy::prelude::*;

// ───── Body ─────────────────────────────────────────────────────────────── //

/// A tile position in the tilemap grid.
#[derive(
    Component,
    Reflect,
    Default,
    Clone,
    Copy,
    Debug,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
)]
#[reflect(Component)]
pub struct TilePos {
    pub x: u32,
    pub y: u32,
}

impl TilePos {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    /// Converts a tile position (2D) into an index in a flattened vector (1D), assuming the
    /// tile position lies in a tilemap of the specified size.
    pub fn to_index(&self, tilemap_size: UVec2) -> usize {
        ((self.y * tilemap_size.x) + self.x) as usize
    }

    /// Checks to see if `self` lies within a tilemap of the specified size.
    pub fn within_map_bounds(&self, map_size: UVec2) -> bool {
        self.x < map_size.x && self.y < map_size.y
    }
}

impl From<TilePos> for UVec2 {
    fn from(pos: TilePos) -> Self {
        UVec2::new(pos.x, pos.y)
    }
}

impl From<&TilePos> for UVec2 {
    fn from(pos: &TilePos) -> Self {
        UVec2::new(pos.x, pos.y)
    }
}

impl From<UVec2> for TilePos {
    fn from(v: UVec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<TilePos> for Vec2 {
    fn from(pos: TilePos) -> Self {
        Vec2::new(pos.x as f32, pos.y as f32)
    }
}

impl From<&TilePos> for Vec2 {
    fn from(pos: &TilePos) -> Self {
        Vec2::new(pos.x as f32, pos.y as f32)
    }
}
