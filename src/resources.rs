use bevy::prelude::*;
use bevy::utils::HashMap;

// ───── Current Crate Imports ────────────────────────────────────────────── //

use crate::TiledComponent;

// ───── Body ─────────────────────────────────────────────────────────────── //

#[derive(Resource, Default)]
pub(crate) struct TiledComponentResource {
    pub(crate) vec: Vec<Box<dyn TiledComponent + Send + Sync>>,
}

impl TiledComponentResource {
    pub(crate) fn new() -> Self {
        TiledComponentResource { vec: vec![] }
    }
    pub(crate) fn insert(
        &mut self,
        component: Box<dyn TiledComponent + Send + Sync>,
    ) {
        self.vec.push(component);
    }
}

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
