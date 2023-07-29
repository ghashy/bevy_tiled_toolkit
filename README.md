# bevy_tiled_toolkit

## Description
This crate is for loading maps from Tiled into Bevy game engine.

## What supported:
- [x] Tile layers
- [x] Object layers
- [x] Tiled animation is played in bevy
- [x] Spritesheet-based tilesets and individual-image-based tilesets
- [x] Spawning custom components with particular tiles entities with `TiledComponent` trait

## What supported with `rapier2d` feature enabled:
- [x] Spawning RigidBody::Fixed with rectangle collision shape
- [x] Spawning RigidBody::Fixed with ball collision shape
- [x] Spawning RigidBody::Fixed with convex polygon collision shape

## What is not supported temporarily:
* Layer offsets
* Layer parallax factors
* Image layers
* Group layers

## What is not supported, because I don't know how to implement it for now:
* Object scaling
