use bevy::asset::{AssetLoader, AssetPath, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::utils::HashMap;
use std::io::Cursor;
use std::rc::Rc;

// ───── Current Crate Imports ────────────────────────────────────────────── //

use super::components::TilemapTexture;

// ───── Body ─────────────────────────────────────────────────────────────── //

type TilesetIdx = usize;

/// Asset, `Handle<TileMap>` we will load from asset_server
#[derive(TypeUuid, TypePath)]
#[uuid = "e51081d0-6168-4881-a1c6-4249b2000d7f"]
pub struct TileMapAsset {
    pub map: tiled::Map,
    pub tilemap_textures: HashMap<TilesetIdx, TilemapTexture>,
    pub atlases: HashMap<TilesetIdx, Handle<TextureAtlas>>,
    pub atlases_offsets: HashMap<TilesetIdx, HashMap<tiled::TileId, usize>>,
    pub tile_image_offsets: HashMap<(TilesetIdx, tiled::TileId), u32>,
}

/// Mock type for piping bytes from `AssetLoader`'s context to
/// `tiled::Loader` for properly parsing `tmx` format.
struct BytesResourceReader {
    bytes: Rc<[u8]>,
}

// Implement `tiled::ResourceReader` to get the ability to call `load_tmx_map`
// function to parse `tmx` file.
impl tiled::ResourceReader for BytesResourceReader {
    type Resource = Cursor<Rc<[u8]>>;
    type Error = std::io::Error;

    fn read_from(
        &mut self,
        _path: &std::path::Path,
    ) -> Result<Self::Resource, Self::Error> {
        // In this case, the path is ignored because the byte data is already
        // provided.
        Ok(Cursor::new(self.bytes.clone()))
    }
}

/// Type for loading `tmx` maps with `bevy`'s `AssetLoader`
pub(crate) struct TiledLoader;

// Loading `TiledMap` asset
impl AssetLoader for TiledLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        Box::pin(async move {
            // Create loader for parsing `tmx` file.
            let mut loader = tiled::Loader::with_cache_and_reader(
                tiled::DefaultResourceCache::new(),
                BytesResourceReader {
                    bytes: Rc::from(bytes),
                },
            );

            // Our parsed map
            let map = loader
                .load_tmx_map(load_context.path())
                .map_err(|e| anyhow::anyhow!("Could not load TMX map: {e}"))?;

            // `dependencies` contains single tile image paths if they are
            // `tilemap_textures` contains textures with idx from enumerate()

            // `tile_image_offsets` contains some strange value: idx from
            // tileset's enumerate(), tile-id and order index of tile
            let (dependencies, tilemap_textures, tile_image_offsets) =
                get_tilemaps_with_deps(&map, load_context);

            let asset_map = TileMapAsset {
                map: map.clone(),
                tilemap_textures,
                atlases: HashMap::new(),
                atlases_offsets: HashMap::new(),
                tile_image_offsets,
            };

            info!("Loaded map: {}", load_context.path().display());

            let loaded_asset = LoadedAsset::new(asset_map);
            load_context.set_default_asset(
                loaded_asset.with_dependencies(dependencies),
            );
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tmx"]
    }
}

fn get_tilemaps_with_deps<'a>(
    map: &tiled::Map,
    load_context: &mut bevy::asset::LoadContext<'_>,
) -> (
    Vec<AssetPath<'a>>,
    HashMap<usize, TilemapTexture>,
    HashMap<(usize, u32), u32>,
) {
    // We will pack into this variables in the next cycle
    let mut dependencies = Vec::new();
    let mut tilemap_textures = HashMap::default();
    let mut tile_image_offsets = HashMap::default();

    // Iterate all tilesets
    for (idx, tileset) in map.tilesets().iter().enumerate() {
        let tilemap_texture = match &tileset.image {
            // If none, tilemap has zero images in `tileset` and one
            // image for each `tile`, handle it.
            None => {
                let mut tile_images: Vec<Handle<Image>> = Vec::new();
                // Fill vec with tiles
                for (id, tile) in tileset.tiles() {
                    if let Some(img) = &tile.image {
                        let tile_path = img.source.clone();
                        let asset_path = AssetPath::new(tile_path, None);
                        info!(
                            "Loading tile image from
                                        {asset_path:?} as image ({id})"
                        );
                        let texture: Handle<Image> =
                            load_context.get_handle(asset_path.clone());
                        tile_image_offsets
                            .insert((idx, id), tile_images.len() as u32);
                        tile_images.push(texture.clone());
                        dependencies.push(asset_path);
                    }
                }
                TilemapTexture::Vector(tile_images)
            }
            Some(img) => {
                let tile_path = img.source.clone();
                let asset_path = AssetPath::new(tile_path, None);
                let texture: Handle<Image> =
                    load_context.get_handle(asset_path.clone());
                dependencies.push(asset_path);

                TilemapTexture::Single(texture.clone())
            }
        };
        tilemap_textures.insert(idx, tilemap_texture);
    }
    // `for`
    (dependencies, tilemap_textures, tile_image_offsets)
}
