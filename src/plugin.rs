use std::time::Duration;

use bevy::asset::*;
use bevy::log;
use bevy::prelude::*;
use bevy::utils::HashMap;

#[cfg(feature = "rapier2d")]
use bevy_rapier2d::prelude::*;

#[cfg(feature = "bevy_ecs_tilemap")]
use bevy_ecs_tilemap::prelude::*;

// ───── Current Crate Imports ────────────────────────────────────────────── //

use crate::asset_loader::TiledLoader;
use crate::asset_loader::TiledMapAsset;
use crate::components::Animation;
use crate::components::LayerStorage;
use crate::components::NeedToSpawn;
use crate::components::TileStorage;
use crate::components::TilesetTexture;
use crate::prelude::DespawnTiledMap;
use crate::prelude::TilePos;
use crate::resources::TiledComponentResource;

// ───── Body ─────────────────────────────────────────────────────────────── //

/// This is a `Bundle` for spawning Tiled tilemap.
#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    /// Main component, contains all information about Tiled map.
    pub tiled_map: Handle<TiledMapAsset>,
    /// Stores all layers entities by name.
    pub layers_storage: LayerStorage,
    /// Stores all tiles entities of all layers of the map.
    pub tile_storage: TileStorage,
    pub name: Name,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed: ComputedVisibility,
}

pub struct TiledToolkitPlugin;

impl Plugin for TiledToolkitPlugin {
    fn build(&self, mut app: &mut App) {
        if !app.is_plugin_added::<TilemapPlugin>() {
            app = app.add_plugins(TilemapPlugin);
        }
        app
            // Custom asset loaders
            .add_asset_loader(TiledLoader)
            // Assets
            .add_asset::<TiledMapAsset>()
            // States
            .add_state::<TiledMapLoadState>()
            // Resources
            .init_resource::<TiledComponentResource>()
            // Systems
            .add_systems(
                Update,
                (
                    system_despawn_maps
                        .run_if(in_state(TiledMapLoadState::Idle)),
                    system_check_asset_state
                        .run_if(in_state(TiledMapLoadState::Idle)),
                    system_setup_atlases
                        .run_if(in_state(TiledMapLoadState::SetupAtlases)),
                    system_process_loaded_maps
                        .run_if(in_state(TiledMapLoadState::Idle)),
                    system_animate_entities,
                )
                    .chain(),
            );
    }
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
enum TiledMapLoadState {
    #[default]
    Idle,
    SetupAtlases,
}

fn system_check_asset_state(
    mut commands: Commands,
    mut tilemap_query: Query<
        (&Handle<TiledMapAsset>, &mut TileStorage, &LayerStorage),
        Without<NeedToSpawn>,
    >,
    mut tilemaps: ResMut<Assets<TiledMapAsset>>,
    mut next_state: ResMut<NextState<TiledMapLoadState>>,
    maps_events: EventReader<AssetEvent<TiledMapAsset>>,
) {
    let changed_maps = events_to_vectors(maps_events);
    let changed_existing = tilemap_query
        .iter_mut()
        .filter(|(handle, _, _)| changed_maps.contains(handle));

    for (tilemap_handle, mut tile_storage, layer_storage) in changed_existing {
        for ecs_storage in tile_storage.bevy_ecs_tilemap_tile_storages.values()
        {
            for tile in ecs_storage.iter().flatten() {
                // In `bevy_ecs_tilamap` there is no point to add childrens to
                // it, they don't have `transform` component. That's why we
                // call `despawn()` instead of `despawn_recursive()`.
                commands.entity(*tile).despawn();
            }
        }
        for tile in tile_storage.iter_all().flatten() {
            commands.entity(*tile).despawn_recursive();
        }
        // Clear storages
        tile_storage.clear();
        tile_storage.bevy_ecs_tilemap_tile_storages.clear();

        for layer in layer_storage.layers.values() {
            // Layer has objects as children, despawn them too.
            commands.entity(*layer).despawn_recursive();
        }

        if let Some(tilemap_asset) = tilemaps.get_mut(tilemap_handle) {
            tilemap_asset.atlases_loaded = false;
        }
        println!("Next state stupatlases");
        next_state.set(TiledMapLoadState::SetupAtlases);
    }
}

/// Slice all textures into atlases
fn system_setup_atlases(
    mut commands: Commands,
    tilemap_query: Query<(Entity, &Handle<TiledMapAsset>)>,
    mut tilemaps: ResMut<Assets<TiledMapAsset>>,
    mut textures: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_assets: ResMut<Assets<TextureAtlas>>,
    mut next_state: ResMut<NextState<TiledMapLoadState>>,
) {
    println!("setuping atlases");
    if tilemap_query.iter().all(|(_, tilemap_handle)| {
        if let Some(tilemap_asset) = tilemaps.get(tilemap_handle) {
            tilemap_asset.atlases_loaded
        } else {
            false
        }
    }) {
        next_state.set(TiledMapLoadState::Idle);
    }

    for (entity, tilemap_handle) in tilemap_query.iter() {
        if !(LoadState::Loaded == asset_server.get_load_state(tilemap_handle)) {
            warn!("TiledMapAsset were not loaded for now");
            continue;
        }
        let Some(tilemap_asset) = tilemaps.get_mut(tilemap_handle) else {
            warn!("No TiledMapAsset existing with that handle!");
            continue;
        };
        if tilemap_asset.atlases_loaded {
            continue;
        }

        // Clear old values after changing
        tilemap_asset.atlases.clear();
        tilemap_asset.atlases_offsets.clear();

        for (tls_idx, tls) in tilemap_asset.map.tilesets().iter().enumerate() {
            // In this case there is expected single spritesheet image
            if let Some(ref tls_image) = tls.image {
                let handle = match tilemap_asset.tilemap_textures.get(&tls_idx)
                {
                    Some(TilesetTexture::Single(handle)) => handle,
                    _ => panic!("Error: tilemap spritesheet was not loaded!"),
                };

                let tile_size =
                    Vec2::new(tls.tile_width as f32, tls.tile_height as f32);
                let columns = tls.columns as usize;
                let padding = Vec2::splat(tls.spacing as f32);
                let rows = ((tls_image.height - tls.margin as i32 * 2)
                    / (tls.tile_height + tls.spacing) as i32)
                    as usize;
                let offset =
                    Vec2::new(tls.offset_x as f32, tls.offset_y as f32);
                let atlas = TextureAtlas::from_grid(
                    handle.clone(),
                    tile_size,
                    columns,
                    rows,
                    Some(padding),
                    Some(offset),
                );
                let handle = texture_atlas_assets.add(atlas);
                tilemap_asset.atlases.insert(tls_idx, handle);
            } else {
                // In this case there is expected vec with individual images
                let handles = match tilemap_asset.tilemap_textures.get(&tls_idx)
                {
                    Some(TilesetTexture::Vector(handles)) => handles,
                    _ => panic!("Error: individual images were not loaded!"),
                };
                // FIXME: detect required size of atlasbuilder
                let mut atlas_builder = TextureAtlasBuilder::default()
                    .max_size(Vec2::new(512. * 20., 512.));
                // Individual image to tile-id offset container
                let offsets = &tilemap_asset.tile_image_offsets;

                // Because of `TextureAtlasBuilder` saves all images in random
                // order, we need to check and save all image offsets in atlas.
                let mut atlas_offsets = Vec::new();
                // Pack images to atlas
                for (tile_id, _) in tls.tiles() {
                    let offset = offsets.get(&(tls_idx, tile_id)).unwrap();
                    let handle = handles.get(*offset as usize).unwrap();
                    let Some(texture) = textures.get(handle) else {
                    warn!("TextureAtlasBuilder: missing image: {:?}.",
                        asset_server.get_handle_path(handle));
                    continue;
                };
                    info!(
                        "Adding texture with offset {}, and id {} to atlas.",
                        offset, tile_id
                    );
                    atlas_builder.add_texture(handle.clone(), texture);
                    atlas_offsets.push((tile_id, handle.clone()));
                }
                let atlas = atlas_builder
                    .finish(&mut textures)
                    .expect("Error: can't build atlas.");

                // Write all atlas offsets to hashmap.
                let mut offsets = HashMap::new();
                for (tile_id, handle) in atlas_offsets {
                    offsets.insert(
                        tile_id,
                        atlas.get_texture_index(&handle).unwrap(),
                    );
                }
                // We can have many individual-image based tilesets.
                tilemap_asset.atlases_offsets.insert(tls_idx, offsets);

                // Store atlas handle with it's tileset index into `tilemap_asset`.
                let handle = texture_atlas_assets.add(atlas);
                tilemap_asset.atlases.insert(tls_idx, handle);
            }
        }
        tilemap_asset.atlases_loaded = true;
        commands.entity(entity).insert(NeedToSpawn);
    }
}

fn system_despawn_maps(
    mut commands: Commands,
    despawned_tilemaps: Query<(Entity, &LayerStorage), With<DespawnTiledMap>>,
) {
    // Despawn tilemaps
    for (entity, layer_storage) in despawned_tilemaps.iter() {
        for layer in layer_storage.layers.values() {
            commands.entity(*layer).despawn_recursive();
        }
        commands.entity(entity).despawn();
    }
}

fn system_process_loaded_maps(
    mut commands: Commands,
    maps: ResMut<Assets<TiledMapAsset>>,
    mut tile_map_query: Query<
        (
            Entity,
            &Handle<TiledMapAsset>,
            &mut TileStorage,
            &mut LayerStorage,
        ),
        With<NeedToSpawn>,
    >,
    asset_server: Res<AssetServer>,
    mut tiled_components: Res<TiledComponentResource>,
) {
    for (map_entity, map_handle, mut tile_storage, mut layer_storage) in
        tile_map_query.iter_mut()
    {
        // If handle is existing, get actual `TiledMap`
        let Some(tilemap_asset) = maps.get(map_handle) else {
            log::warn!("Cant get tiled_map from Assets<TiledMap>!");
            continue;
        };

        // Iterate over layers
        for (layer_idx, layer) in tilemap_asset.map.layers().enumerate() {
            let layer_entity = spawn_layer(
                layer,
                layer_idx,
                &mut commands,
                &asset_server,
                tilemap_asset,
                &mut tiled_components,
                &mut tile_storage,
            );
            let layer_name = Name::new(layer.name.clone());

            layer_storage
                .layers
                .insert(layer_name.clone(), layer_entity);
            commands.entity(layer_entity).insert(layer_name);
            commands
                .entity(map_entity)
                .push_children(&[layer_entity])
                .remove::<NeedToSpawn>();
        }
    }
}

fn spawn_with_bevy_ecs_tilemap(
    commands: &mut Commands,
    layer: &tiled::Layer,
    layer_idx: usize,
    tilemap_asset: &TiledMapAsset,
    tile_storage: &mut TileStorage,
) -> Entity {
    let layer_entity = commands.spawn_empty().id();
    let layer_opacity = layer.opacity;
    match layer.layer_type() {
        tiled::LayerType::Tiles(layer) => match layer {
            tiled::TileLayer::Infinite(_) => {
                panic!("Infinite layers unsupported!")
            }
            tiled::TileLayer::Finite(layer_data) => {
                let tile_width = tilemap_asset.map.tile_width as i32;
                let tile_height = tilemap_asset.map.tile_height as i32;
                let layer_tile = match get_first_tile(
                    layer.width().unwrap() as i32,
                    layer.height().unwrap() as i32,
                    layer_data,
                ) {
                    Some(t) => t,
                    None => {
                        // Skip empty tile
                        println!("Skipping empty tile");
                        return layer_entity;
                    }
                };
                let tls_idx = layer_tile.tileset_index();

                let tileset_texture =
                    match tilemap_asset.tilemap_textures.get(&tls_idx) {
                        Some(t) => t.clone(),
                        None => {
                            error!(
                                "There are no atlas for tilemap with index {}",
                                tls_idx
                            );
                            return layer_entity;
                        }
                    };
                let map_size = TilemapSize {
                    x: layer_data.width(),
                    y: layer_data.height(),
                };
                let mut ecs_tile_storage =
                    bevy_ecs_tilemap::prelude::TileStorage::empty(map_size);

                for x in 0..map_size.x {
                    for y in 0..map_size.y {
                        // Transform TMX coords into bevy coords.
                        let mapped_y = tilemap_asset.map.height - 1 - y;

                        let mapped_x = x as i32;
                        let mapped_y = mapped_y as i32;

                        let layer_tile =
                            match layer_data.get_tile(mapped_x, mapped_y) {
                                Some(t) => t,
                                None => {
                                    // Skip empty tile
                                    continue;
                                }
                            };
                        let texture_index = match tileset_texture {
                            TilesetTexture::Single(_) => layer_tile.id(),
                            TilesetTexture::Vector(_) =>
                            *tilemap_asset.tile_image_offsets.get(&(tls_idx, layer_tile.id()))
                            .expect("The offset into to image vector should have been saved during the initial load."),
                        };
                        let tile_pos =
                            bevy_ecs_tilemap::prelude::TilePos { x, y };
                        let tile_entity = commands
                            .spawn(TileBundle {
                                position: tile_pos,
                                tilemap_id: TilemapId(layer_entity),
                                texture_index: TileTextureIndex(texture_index),
                                color: TileColor(
                                    Color::WHITE.with_a(layer_opacity),
                                ),
                                ..default()
                            })
                            .id();
                        ecs_tile_storage.set(&tile_pos, tile_entity);
                    }
                }

                let texture = match tileset_texture {
                    TilesetTexture::Single(img) => TilemapTexture::Single(img),
                    TilesetTexture::Vector(v) => TilemapTexture::Vector(v),
                };

                let tile_size = TilemapTileSize {
                    x: tile_width as f32,
                    y: tile_height as f32,
                };
                let grid_size = tile_size.into();
                let map_type = TilemapType::default();

                commands
                    .entity(layer_entity)
                    .insert(TilemapBundle {
                        grid_size,
                        map_type,
                        size: map_size,
                        storage: ecs_tile_storage.clone(),
                        texture,
                        tile_size,
                        transform: Transform::from_xyz(
                            tile_width as f32 * 0.5,
                            tile_height as f32 * 0.5,
                            layer_idx as f32,
                        ),
                        ..default()
                    })
                    .push_children(
                        &ecs_tile_storage
                            .iter()
                            .flatten()
                            .map(|&e| e)
                            .collect::<Vec<_>>()[..],
                    );
                tile_storage
                    .bevy_ecs_tilemap_tile_storages
                    .insert(layer_idx, ecs_tile_storage);
            }
        },
        _ => error!("bevy_ecs_tilemap supports only LayerType::Tiles layers!"),
    }
    layer_entity
}

fn spawn_layer(
    layer: tiled::Layer,
    layer_idx: usize,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    tilemap_asset: &TiledMapAsset,
    tiled_components: &mut Res<TiledComponentResource>,
    tile_storage: &mut TileStorage,
) -> Entity {
    for (k, v) in &layer.properties {
        if k == "bevy_ecs_tilemap" {
            if let tiled::PropertyValue::BoolValue(needs) = v {
                if *needs {
                    return spawn_with_bevy_ecs_tilemap(
                        commands,
                        &layer,
                        layer_idx,
                        tilemap_asset,
                        tile_storage,
                    );
                }
            }
        }
    }
    let layer_entity = commands
        .spawn((SpatialBundle {
            transform: Transform::from_xyz(0., 0., layer_idx as f32),
            ..default()
        },))
        .id();
    let layer_opacity = layer.opacity;
    match layer.layer_type() {
        tiled::LayerType::Tiles(layer) => {
            match layer {
                tiled::TileLayer::Infinite(_) => {
                    panic!("Infinite layers unsupported!")
                }
                tiled::TileLayer::Finite(layer) => {
                    let map_width = layer.width() as i32;
                    let map_height = layer.height() as i32;
                    let tile_width = tilemap_asset.map.tile_width as i32;
                    let tile_height = tilemap_asset.map.tile_height as i32;
                    match tile_storage.init_place(
                        layer_idx,
                        UVec2::new(layer.width(), layer.height()),
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Error: {}", e);
                        }
                    }

                    match tilemap_asset.map.orientation {
                        tiled::Orientation::Orthogonal => {
                            for x in 0..map_width {
                                for y in 0..map_height {
                                    let layer_tile = match layer.get_tile(x, y)
                                    {
                                        Some(t) => t,
                                        None => {
                                            // Skip empty tile
                                            continue;
                                        }
                                    };
                                    // Transform TMX coords into bevy coords.
                                    let mapped_y =
                                        tilemap_asset.map.height - 1 - y as u32;
                                    let mapped_x = x;
                                    let mapped_y = mapped_y as i32;

                                    let tls_idx = layer_tile.tileset_index();
                                    let layer_tile_data =
                                        match layer.get_tile_data(x, y) {
                                            Some(t) => t,
                                            None => continue,
                                        };
                                    let tile = match layer_tile.get_tile() {
                                        Some(t) => t,
                                        None => continue,
                                    };
                                    let texture_atlas = match tilemap_asset
                                        .atlases
                                        .get(&tls_idx)
                                    {
                                        Some(t) => t.clone(),
                                        None => {
                                            error!("There are no atlas for tilemap with index {}", tls_idx);
                                            continue;
                                        }
                                    };

                                    // Spawn tile
                                    let mut tile_entity_commands = commands
                                        .spawn(SpriteSheetBundle {
                                            transform: Transform::from_xyz(
                                                (mapped_x * tile_width) as f32
                                                    + tile_width as f32 * 0.5,
                                                (mapped_y * tile_height) as f32
                                                    + tile_height as f32 * 0.5,
                                                1.,
                                            ),
                                            sprite: TextureAtlasSprite {
                                                index: layer_tile_data.id()
                                                    as usize,
                                                flip_x: layer_tile_data.flip_h,
                                                flip_y: layer_tile_data.flip_v,
                                                color: Color::WHITE
                                                    .with_a(layer_opacity),
                                                ..default()
                                            },
                                            texture_atlas,
                                            ..default()
                                        });

                                    spawn_tiled_components(
                                        &tile,
                                        tiled_components,
                                        &mut tile_entity_commands,
                                        asset_server,
                                    );

                                    let tile_entity = tile_entity_commands.id();

                                    add_animation_if_needed(
                                        &tile,
                                        tilemap_asset,
                                        &tls_idx,
                                        commands,
                                        tile_entity,
                                    );

                                    add_rigidbodies_if_needed(
                                        &tile,
                                        commands,
                                        tile_entity,
                                        tile_width as f32,
                                        tile_height as f32,
                                    );

                                    commands
                                        .entity(layer_entity)
                                        .add_child(tile_entity);

                                    // INSPECT: Tiled x and y or bevy-mapped?
                                    // Leave Tiled for now
                                    match tile_storage.set(
                                        layer_idx,
                                        &TilePos::new(x as u32, y as u32),
                                        tile_entity,
                                    ) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!("Error: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            panic!("Only orthogonal maps supported!");
                        }
                    }
                }
            }
        }
        tiled::LayerType::Objects(layer) => {
            for obj in layer.objects() {
                let Some(tile_data) = obj.tile_data() else {
                    warn!("No tile data for obj {:?}", obj);
                    continue;
                };
                let Some(tile) = obj.get_tile() else {
                    warn!("No tile for obj {:?}", obj);
                    continue;
                };
                let tls_idx = match tile_data.tileset_location() {
                    tiled::TilesetLocation::Map(idx) => idx,
                    tiled::TilesetLocation::Template(_) => {
                        error!("Tileset for object was from Template!");
                        continue;
                    }
                };
                let texture_atlas = match tilemap_asset.atlases.get(tls_idx) {
                    Some(t) => t.clone(),
                    None => {
                        error!(
                            "There are no atlas for tilemap with index {}",
                            tls_idx
                        );
                        continue;
                    }
                };

                let obj_width = if let Some(tile) = tile.get_tile() {
                    if let Some(ref image) = tile.image {
                        image.width as f32
                    } else {
                        tile.tileset().tile_width as f32
                    }
                } else {
                    tile.get_tileset().tile_width as f32
                };

                let obj_height = if let Some(tile) = tile.get_tile() {
                    if let Some(ref image) = tile.image {
                        image.height as f32
                    } else {
                        tile.tileset().tile_height as f32
                    }
                } else {
                    tile.get_tileset().tile_height as f32
                };

                let map_height = (tilemap_asset.map.height
                    * tilemap_asset.map.tile_height)
                    as f32;

                let mapped_x = obj.x + obj_width * 0.5;
                let mapped_y = map_height - obj.y + obj_height * 0.5;

                // Spawn object
                let mut obj_entity_commands =
                    commands.spawn(SpriteSheetBundle {
                        transform: Transform::from_xyz(mapped_x, mapped_y, 1.),
                        sprite: TextureAtlasSprite {
                            index: tile.id() as usize,
                            flip_x: tile.flip_h,
                            flip_y: tile.flip_v,
                            color: Color::WHITE.with_a(layer_opacity),
                            ..default()
                        },
                        texture_atlas,
                        ..default()
                    });

                let obj_entity = obj_entity_commands.id();

                if let Some(tile) = obj.get_tile() {
                    if let Some(ref tile) = tile.get_tile() {
                        // Handle custom components
                        spawn_tiled_components(
                            &tile,
                            tiled_components,
                            &mut obj_entity_commands,
                            asset_server,
                        );
                        // Handle animation
                        add_animation_if_needed(
                            tile,
                            tilemap_asset,
                            tls_idx,
                            commands,
                            obj_entity,
                        );
                        // Handle collision
                        add_rigidbodies_if_needed(
                            tile, commands, obj_entity, obj_width, obj_height,
                        );
                    }
                };

                commands.entity(layer_entity).add_child(obj_entity);
            }
        }
        tiled::LayerType::Image(layer) => {
            // Spawn image layer
            todo!()
        }
        tiled::LayerType::Group(layer) => {
            // Spawn group layer
            todo!()
        }
    };
    layer_entity
}

fn spawn_tiled_components(
    tile: &tiled::Tile,
    tiled_components: &mut Res<TiledComponentResource>,
    tile_entity_commands: &mut bevy::ecs::system::EntityCommands,
    asset_server: &Res<AssetServer>,
) {
    let properties: HashMap<String, tiled::PropertyValue> = tile
        .properties
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    for comp in &tiled_components.vec {
        if let Some(ref class) = tile.user_type {
            if comp.get_class_name() == class {
                comp.insert_self_to_entity(
                    tile_entity_commands,
                    properties.clone(),
                    asset_server,
                );
            }
        }
    }
}

fn add_rigidbodies_if_needed(
    tile: &tiled::Tile,
    commands: &mut Commands,
    entity: Entity,
    container_width: f32,
    container_height: f32,
) {
    if let Some(ref obj_layer_data) = tile.collision {
        for data in obj_layer_data.object_data() {
            use tiled::ObjectShape;
            match &data.shape {
                ObjectShape::Rect { width, height } => {
                    commands
                        .entity(entity)
                        .insert(RigidBody::Fixed)
                        .with_children(|parent| {
                            let mapped_x_zero =
                                container_width / 2. - width / 2.;
                            let x_tiled_to_bevy =
                                (mapped_x_zero - data.x) * -1.;
                            let mapped_y_zero =
                                container_height / 2. - height / 2.;
                            let y_tiled_to_bevy = mapped_y_zero - data.y;
                            parent.spawn((
                                Collider::cuboid(*width * 0.5, *height * 0.5),
                                Transform::from_xyz(
                                    x_tiled_to_bevy,
                                    y_tiled_to_bevy,
                                    0.,
                                ),
                            ));
                        });
                }
                ObjectShape::Ellipse { width, height } => {
                    if width != height {
                        error!(
                            "Only ball colliders supported! Spawning ball instead of ellipse."
                        );
                    }
                    commands
                        .entity(entity)
                        .insert(RigidBody::Fixed)
                        .with_children(|parent| {
                            let mapped_x_zero =
                                container_width / 2. - width / 2.;
                            let x_tiled_to_bevy =
                                (mapped_x_zero - data.x) * -1.;
                            let mapped_y_zero =
                                container_height / 2. - height / 2.;
                            let y_tiled_to_bevy = mapped_y_zero - data.y;
                            parent.spawn((
                                Collider::ball(*width * 0.5),
                                Transform::from_xyz(
                                    x_tiled_to_bevy,
                                    y_tiled_to_bevy,
                                    0.,
                                ),
                            ));
                        });
                }
                ObjectShape::Polygon { points } => {
                    let points = points
                        .iter()
                        .map(|(x, y)| Vec2::new(*x, *y * -1.))
                        .collect::<Vec<Vec2>>();
                    let collider = Collider::convex_hull(&points).unwrap();
                    let mapped_x_zero = container_width / 2.;
                    let x_tiled_to_bevy = (mapped_x_zero - data.x) * -1.;
                    let mapped_y_zero = container_height / 2.;
                    let y_tiled_to_bevy = mapped_y_zero - data.y;
                    commands
                        .entity(entity)
                        .insert(RigidBody::Fixed)
                        .with_children(|parent| {
                            parent.spawn((
                                collider,
                                Transform::from_xyz(
                                    x_tiled_to_bevy,
                                    y_tiled_to_bevy,
                                    0.,
                                ),
                            ));
                        });
                }
                _ => {
                    panic!("Not implemented");
                }
            }
        }
    }
}

fn add_animation_if_needed(
    tile: &tiled::Tile,
    tilemap_asset: &TiledMapAsset,
    tls_idx: &usize,
    commands: &mut Commands,
    entity: Entity,
) {
    if let Some(ref frames) = tile.animation {
        if let Some(frame) = frames.first() {
            let atlas_offsets = match tilemap_asset.atlases_offsets.get(tls_idx)
            {
                // Tiles packed into atlas are unordered, we need offsets
                Some(ofsts) => ofsts.clone(),
                // If there are no offsets, it means that all tiles are ordered
                // and we will use tile-id as offsets.
                None => HashMap::new(),
            };
            let timer = Timer::new(
                Duration::from_millis(frame.duration as u64),
                TimerMode::Repeating,
            );
            commands.entity(entity).insert((Animation {
                frames: frames.clone(),
                current_frame: 0,
                offsets: atlas_offsets,
                timer,
            },));
        }
    }
}

fn system_animate_entities(
    mut query: Query<(&mut Animation, &mut TextureAtlasSprite)>,
    time: Res<Time>,
) {
    for (mut animation, mut atlas) in query.iter_mut() {
        if animation.timer.tick(time.delta()).just_finished() {
            let fr_idx = inc_frame(
                animation.current_frame,
                animation.frames.len() as u32 - 1,
            );
            let tile_id =
                animation.frames.get(fr_idx as usize).unwrap().tile_id;
            animation.current_frame = fr_idx;
            atlas.index = match animation.offsets.get(&tile_id) {
                // Atlas was created from tiles, (unordered tiles)
                Some(v) => *v,
                // Atlas was loaded from image, (ordered tiles)
                None => tile_id as usize,
            };
            let fr_dur =
                animation.frames.get(fr_idx as usize).unwrap().duration;
            animation
                .timer
                .set_duration(Duration::from_millis(fr_dur as u64));
        }
    }
}

// ───── Utility functions ────────────────────────────────────────────────── //

fn events_to_vectors(
    mut maps_events: EventReader<AssetEvent<TiledMapAsset>>,
) -> Vec<Handle<TiledMapAsset>> {
    let mut changed_maps = Vec::<Handle<TiledMapAsset>>::default();
    for event in maps_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                log::info!("Map added!");
                // Handle of event is already Weak
                changed_maps.push(handle.clone_weak());
            }
            AssetEvent::Modified { handle } => {
                log::info!("Map modified!");
                changed_maps.push(handle.clone_weak());
            }
            AssetEvent::Removed { handle } => {
                log::info!("Map removed!");
            }
        }
    }
    changed_maps
}

fn tiled_color_to_bevy(color: &tiled::Color) -> Color {
    let red = color.red as f32 / 255.;
    let green = color.green as f32 / 255.;
    let blue = color.blue as f32 / 255.;
    let alpha = color.alpha as f32 / 255.;
    Color::Rgba {
        red,
        green,
        blue,
        alpha,
    }
}

fn inc_frame(cur: u32, max: u32) -> u32 {
    if cur >= max {
        0
    } else {
        cur + 1
    }
}

fn get_first_tile(
    layer_width: i32,
    layer_height: i32,
    layer: tiled::FiniteTileLayer,
) -> Option<tiled::LayerTile> {
    for x in 0..layer_width {
        for y in 0..layer_height {
            if let Some(layer_tile) = layer.get_tile(x, y) {
                return Some(layer_tile);
            }
        }
    }
    None
}

#[allow(dead_code)]
fn get_rect_from_convex(vec: &[Vec2]) -> (f32, f32) {
    let mut x_min = vec[0].x;
    let mut y_min = vec[0].y;
    let mut x_max = vec[0].x;
    let mut y_max = vec[0].y;

    for point in vec {
        if point.x < x_min {
            x_min = point.x
        }
        if point.y < y_min {
            y_min = point.y
        }
        if point.x > x_max {
            x_max = point.x
        }
        if point.y > y_max {
            y_max = point.y
        }
    }

    (x_max - x_min, y_max - y_min)
}
