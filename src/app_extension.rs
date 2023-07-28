use bevy::prelude::*;

// ───── Current Crate Imports ────────────────────────────────────────────── //

use crate::{resources::TiledComponentResource, TiledComponent};

// ───── Body ─────────────────────────────────────────────────────────────── //

pub trait TiledComponentReg {
    fn register_tiled_component<T>(&mut self) -> &mut Self
    where
        T: TiledComponent + Default + Send + Sync + 'static;
}

impl TiledComponentReg for App {
    fn register_tiled_component<T>(&mut self) -> &mut Self
    where
        T: TiledComponent + Default + Send + Sync + 'static,
    {
        match self.world.get_resource_mut::<TiledComponentResource>() {
            Some(mut res) => {
                res.vec.push(Box::new(T::default()));
            }
            None => {
                let mut res = TiledComponentResource::new();
                res.vec.push(Box::new(T::default()));
                self.world.insert_resource::<TiledComponentResource>(res);
            }
        }
        self
    }
}
