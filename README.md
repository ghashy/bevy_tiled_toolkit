This is a [Tiled](https://www.mapeditor.org) integration plugin for
[Bevy](https://bevyengine.org) game engine.

### Getting starged
This plugin allows to use almost every features from `Tiled` map editor in
in your Bevy-game.

Follow these steps to start:
1. Add the [TiledToolkitPlugin](self::plugin::TiledToolkitPlugin) to the `App`.
2. Spawn a [TiledMapBundle](self::plugin::TiledMapBundle):
```rust
fn system_spawn_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        let tiled_map: Handle<TiledMapAsset> =
           asset_server.load("tiled/tilemaps/Map.tmx");

        commands.spawn(TiledMapBundle {
            tiled_map,
            name: Name::from("TiledMap"),
            ..default()
        });
     }
}
```
If you want to spawn your custom component with particular tile, follow
these instructions:
1. Declare your custom component:
```rust
#[derive(Component, Default)]
struct Ninja {
    strength: f32,
}
```
2. Implement `TiledComponent` trait for your type:
```rust
impl TiledComponent for Ninja {
    fn insert_self_to_entity(
        &self,
        commands: &mut EntityCommands,
        values: HashMap<String, tiled::PropertyValue>,
        asset_server: &Res<AssetServer>,
    ) {
        for (key, value) in values {
            if key == String::from("strength") {
                let tiled::PropertyValue::FloatValue(v) = value else {
                    error!("Cant spawn Ninja, wrong PropertyValue type");
                    continue;
                };
                commands.insert(Ninja { strength: v });
            }
        }
    }
    fn get_class_name(&self) -> &str {
        "Ninja"
    }
}
```
3. Register your type in the `App`:
```rust
app.register_tiled_component::<Ninja>()
```
4. In `Tiled`, in the `Class` field of your tile, insert the same name
that `get_class_name` function returns.
5. Create properties in `Tiled` for your tile, and they will be passed to
`insert_self_to_entity` function, where you can use them to initialize your
component.

### Rendering with bevy_ecs_tilemap

This crate supports rendering layers with [bevy_ecs_tilemap](https://github.com/StarArawn/bevy_ecs_tilemap),
but there are some limitations:
* Only tile layers supported.
* Each tile should have the same size.
* One layer should use only one tilemap at the same time.

At the same time it is recommended to render at least the base layer with
[bevy_ecs_tilemap](https://github.com/StarArawn/bevy_ecs_tilemap)
(because base layer usually completely filled with sprites).

If you spawn each tile as just `TextureAtlas`'es on such layer, perfomance will
be poor on mobile devices and low-end computers, especially on medium-sized
and big-sized maps.
There will be tile-flicker when moving camera on iphone 8, if map is bigger then 50x50.
`TextureAtlas`'es based rendering works good for tile layers, where there
are not too many tiles, and, naturally, `Object layers` can be rendered
only with `TextureAtlas`'es.

To enable `bevy_ecs_tilemap` rendering on the particular layer, you should
create boolean property named `bevy_ecs_tilemap` on the desired layer in
Tiled, and click on checkbox of this property to activate it.

### Despawning tilemap
Spawn a [DespawnTilemap](self::components::DespawnTiledMap) component to an
entity with `Handle<TiledMapAsset>` to despawn the tilemap:
```rust
fn system_despawn_map(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    tiled_map_query: Query<Entity, With<Handle<TiledMapAsset>>>,
) {
    if input.just_pressed(KeyCode::P) {
        for entity in tiled_map_query.iter() {
            commands.entity(entity).insert(DespawnTiledMap);
        }
    }
}
```

## What supported:
- [x] Tile layers
- [x] Object layers
- [x] Tiled animation is played in bevy
- [x] Spritesheet-based tilesets and individual-image-based tilesets
- [x] Spawning custom components with particular tiles entities with `TiledComponent` trait
- [x] Maps with orthogonal orientation

## What supported with `rapier2d` feature enabled:
- [x] Spawning RigidBody::Fixed with rectangle collision shape
- [x] Spawning RigidBody::Fixed with ball collision shape
- [x] Spawning RigidBody::Fixed with convex polygon collision shape

## What is not supported temporarily:
* Isometric, Staggered and Hexagonal maps
* Layer offsets
* Layer parallax factors
* Image layers
* Group layers

## What is not supported, because I don't know how to implement it for now:
* Filesystem watcher feature does not work properly with this plugin
* Object scaling
* Infinite tile layers
