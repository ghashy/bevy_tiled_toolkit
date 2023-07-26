use bevy::prelude::*;
use bevy::utils::HashMap;

// ───── Body ─────────────────────────────────────────────────────────────── //

#[derive(Default, Debug, Clone)]
pub struct TiledPoint {
    #[allow(dead_code)]
    x: f32,
    #[allow(dead_code)]
    y: f32,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct TiledPoints {
    #[allow(dead_code)]
    points: HashMap<String, TiledPoint>,
}
