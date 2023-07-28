use std::time::Duration;

use bevy::asset::*;
use bevy::log;
use bevy::prelude::*;
use bevy::utils::HashMap;

#[cfg(feature = "rapier2d")]
use bevy_rapier2d::prelude::*;

// ───── Current Crate Imports ────────────────────────────────────────────── //

use crate::asset_loader::TiledLoader;
use crate::asset_loader::TilemapAsset;
use crate::components::Animation;
use crate::components::LayerStorage;
use crate::components::TileStorage;
use crate::components::TilesetTexture;
use crate::resources::TiledComponentResource;

// ───── Body ─────────────────────────────────────────────────────────────── //

/// This is a `Bundle` for spawning tiled tilemap.
#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    pub tiled_map: Handle<TilemapAsset>,
    pub transform: Transform,
    pub layers_storage: LayerStorage,
    pub tile_storage: TileStorage,
    pub global_transform: GlobalTransform,
}

pub struct TiledToolkitPlugin;

impl Plugin for TiledToolkitPlugin {
    fn build(&self, app: &mut App) {
        app
            // Custom asset loaders
            .add_asset_loader(TiledLoader)
            // Assets
            .add_asset::<TilemapAsset>()
            // States
            .add_state::<TilemapLoadState>()
            // Resources
            .init_resource::<TiledComponentResource>()
            .insert_resource(Msaa::Off)
            // Systems
            .add_systems(
                Update,
                (
                    listen_for_tilemap_loading
                        .run_if(in_state(TilemapLoadState::Idle)),
                    check_tilemap_load_state
                        .run_if(in_state(TilemapLoadState::Loading)),
                    setup_atlases
                        .run_if(in_state(TilemapLoadState::SetupAtlases)),
                    system_process_loaded_maps
                        .run_if(in_state(TilemapLoadState::Loaded)),
                    animate_entities.run_if(in_state(TilemapLoadState::Loaded)),
                ),
            );
    }
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
enum TilemapLoadState {
    #[default]
    Idle,
    Loading,
    SetupAtlases,
    Loaded,
}

fn listen_for_tilemap_loading(
    mut next_state: ResMut<NextState<TilemapLoadState>>,
    tilemap: Query<Added<Handle<TilemapAsset>>>,
) {
    if let Some(_) = tilemap.iter().next() {
        next_state.set(TilemapLoadState::Loading);
    }
}

fn check_tilemap_load_state(
    mut next_state: ResMut<NextState<TilemapLoadState>>,
    tilemap: Query<&Handle<TilemapAsset>>,
    asset_server: Res<AssetServer>,
) {
    if tilemap.iter().count() != 1 {
        panic!("Expected to have only 1 tilemap loading at once!");
    }
    if let LoadState::Loaded =
        asset_server.get_load_state(tilemap.iter().next().unwrap())
    {
        next_state.set(TilemapLoadState::SetupAtlases);
    }
}

fn setup_atlases(
    tilemap_handle: Query<&Handle<TilemapAsset>>,
    mut tilemaps: ResMut<Assets<TilemapAsset>>,
    mut textures: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<TilemapLoadState>>,
    mut texture_atlas_assets: ResMut<Assets<TextureAtlas>>,
) {
    let tilemap_asset = tilemaps.get_mut(tilemap_handle.single()).unwrap();
    for (tls_idx, tls) in tilemap_asset.map.tilesets().iter().enumerate() {
        // In this case there is expected single spritesheet image
        if let Some(ref tls_image) = tls.image {
            let handle = match tilemap_asset.tilemap_textures.get(&tls_idx) {
                Some(TilesetTexture::Single(handle)) => handle,
                _ => panic!("Error: expected single image"),
            };

            let tile_size =
                Vec2::new(tls.tile_width as f32, tls.tile_height as f32);
            let columns = tls.columns as usize;
            let padding = Vec2::splat(tls.spacing as f32);
            let rows = ((tls_image.height - tls.margin as i32 * 2)
                / (tls.tile_height + tls.spacing) as i32)
                as usize;
            info!("Detected {rows} rows in tileset: {}", tls.name);
            let offset = Vec2::new(tls.offset_x as f32, tls.offset_y as f32);

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
            let handles = match tilemap_asset.tilemap_textures.get(&tls_idx) {
                Some(TilesetTexture::Vector(handles)) => handles,
                _ => panic!("Error: expected vector of images"),
            };
            let mut atlas_builder = TextureAtlasBuilder::default()
                .max_size(Vec2::new(512. * 20., 512.));
            // let image_count = handles.len();
            let offsets = &tilemap_asset.tile_image_offsets;
            // TODO: doc this
            let mut atlas_offsets = Vec::new();

            for (tile_id, _) in tls.tiles() {
                let offset = offsets.get(&(tls_idx, tile_id)).unwrap();
                let handle = handles.get(*offset as usize).unwrap();
                let Some(texture) = textures.get(handle) else {
                    warn!("There are no {:?} image", asset_server.get_handle_path(handle));
                    continue;
                };
                info!(
                    "Adding texture with offset {}, and id {}",
                    offset, tile_id
                );
                atlas_builder.add_texture(handle.clone(), texture);
                atlas_offsets.push((tile_id, handle.clone()));
            }
            let atlas = atlas_builder
                .finish(&mut textures)
                .expect("Error: cant build atlas");

            // TODO: doc this
            let mut offsets = HashMap::new();
            for (tile_id, handle) in atlas_offsets {
                offsets
                    .insert(tile_id, atlas.get_texture_index(&handle).unwrap());
            }
            tilemap_asset.atlases_offsets.insert(tls_idx, offsets);

            // TODO: doc this
            let handle = texture_atlas_assets.add(atlas);
            tilemap_asset.atlases.insert(tls_idx, handle);
        }
    }
    next_state.set(TilemapLoadState::Loaded);
}

fn system_process_loaded_maps(
    mut commands: Commands,
    maps_events: EventReader<AssetEvent<TilemapAsset>>,
    maps: ResMut<Assets<TilemapAsset>>,
    mut tile_map_query: Query<(
        &Handle<TilemapAsset>,
        &mut TileStorage,
        &mut LayerStorage,
    )>,
    asset_server: Res<AssetServer>,
    mut tiled_components: Res<TiledComponentResource>,
) {
    let changed_maps = events_to_vectors(maps_events);

    // Iter with changed maps, only existing in World for this update
    let changed_existing = tile_map_query
        .iter_mut()
        .filter(|(handle, _, _)| changed_maps.contains(handle));

    for (map_handle, mut tile_storage, mut layer_storage) in changed_existing {
        // Clear storages of the map
        tile_storage.clear();
        layer_storage.layers.clear();

        // If handle is existing, get actual `TiledMap`
        if let Some(tilemap_asset) = maps.get(map_handle) {
            // Iterate over layers
            for (layer_idx, layer) in tilemap_asset.map.layers().enumerate() {
                let layer_entity = spawn_layer(
                    layer,
                    layer_idx,
                    &mut commands,
                    &asset_server,
                    tilemap_asset,
                    &mut tiled_components,
                );
                let layer_name = Name::new(layer.name.clone());

                layer_storage
                    .layers
                    .insert(layer_name.clone(), layer_entity);
                commands.entity(layer_entity).insert(layer_name);
            }
        } else {
            log::warn!("Cant get tiled_map from Assets<TiledMap>!");
        }
    }
}

fn spawn_layer(
    layer: tiled::Layer,
    layer_idx: usize,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    tilemap_asset: &TilemapAsset,
    tiled_components: &mut Res<TiledComponentResource>,
) -> Entity {
    let layer_entity = commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(0., 0., layer_idx as f32),
                ..default()
            },
            Name::from(layer.name.clone()),
        ))
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

                                    // commands
                                    //     .entity(tile_entity)
                                    //     .insert(Tile { properties });

                                    commands
                                        .entity(layer_entity)
                                        .add_child(tile_entity);
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
    tilemap_asset: &TilemapAsset,
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

fn animate_entities(
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
    mut maps_events: EventReader<AssetEvent<TilemapAsset>>,
) -> Vec<Handle<TilemapAsset>> {
    let mut changed_maps = Vec::<Handle<TilemapAsset>>::default();
    for event in maps_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                log::info!("Map added!");
                // Handle of event is already Weak
                changed_maps.push(handle.clone_weak());
            }
            AssetEvent::Modified { handle } => {
                log::info!("Map changed!");
                changed_maps.push(handle.clone_weak());
            }
            AssetEvent::Removed { handle: _ } => {
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
