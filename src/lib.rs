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

// Top-level modules
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
    pub use crate::plugin::TiledMapBundle;
    pub use crate::plugin::TiledToolkitPlugin;
}
