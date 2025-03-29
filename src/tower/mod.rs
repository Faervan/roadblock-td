use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use bevy::prelude::*;
use placing::TowerPlacingPlugin;

pub use placing::{SelectedTower, TowerPlaceState, place_tower};

use crate::{
    Orientation,
    grid::{Grid, GridPos, TILE_SIZE},
};

mod placing;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<TowerPlaceState>();
        app.register_type::<Tower>();
        app.add_plugins(TowerPlacingPlugin);
    }
}

#[derive(Reflect, Component, Clone, Copy)]
#[reflect(Component)]
pub struct Tower {
    pub variant: TowerType,
    orientation: Orientation,
}

#[derive(Reflect, Clone, Copy)]
pub enum TowerType {
    Wall,
    SpikedWall,
    Canon,
}

impl Tower {
    pub fn new(variant: TowerType, orientation: Orientation) -> Self {
        Self {
            variant,
            orientation,
        }
    }

    fn fill_grid(&self, origin: &GridPos, grid: &mut Grid, entity: Entity) -> Vec<GridPos> {
        let mut blocked = vec![];
        grid.tower_origins.insert(entity, *origin);
        // Add entity to every coordinate it covers
        let (rows, cols) = self.size();
        for i in 0..rows {
            for j in 0..cols {
                let pos = GridPos::new(origin.row + j, origin.col + i);
                grid.tower.insert(pos, entity);
                blocked.push(pos);
            }
        }
        blocked
    }

    pub fn clear_grid(&self, grid: &mut Grid, entity: Entity) -> Vec<GridPos> {
        let mut freed = vec![];
        let Some(origin) = grid.tower_origins.remove(&entity) else {
            return vec![];
        };
        let (rows, cols) = self.size();
        for i in 0..rows {
            for j in 0..cols {
                let pos = GridPos::new(origin.row + j, origin.col + i);
                grid.tower.remove(&pos);
                freed.push(pos);
            }
        }
        freed
    }

    pub fn size(&self) -> (isize, isize) {
        let size = self.variant.size();
        // Flip Dimensions of the tower in case of rotation
        match self.orientation.is_horizontal() {
            true => (size.1, size.0),
            false => size,
        }
    }

    fn offset(&self) -> (isize, isize) {
        match self.variant {
            TowerType::Wall => (1, 0),
            TowerType::SpikedWall => (1, 0),
            TowerType::Canon => (1, 1),
        }
    }
}

impl TowerType {
    //temp values as balancing cannot happen until a basic gameplay loop is in place
    fn max_hp(&self) -> isize {
        match self {
            TowerType::Wall => 100,
            TowerType::SpikedWall => 100,
            TowerType::Canon => 80,
        }
    }

    pub fn size(&self) -> (isize, isize) {
        match self {
            TowerType::Wall => (4, 1),
            TowerType::SpikedWall => (4, 1),
            TowerType::Canon => (3, 3),
        }
    }

    fn _range(&self) -> f32 {
        match self {
            TowerType::Canon => TILE_SIZE * 10.0,
            _ => 0.0,
        }
    }

    fn _strength(&self) -> u32 {
        match self {
            TowerType::Canon => 15,
            _ => 0,
        }
    }

    fn _fire_cooldown(&self) -> Duration {
        match self {
            TowerType::Canon => Duration::from_secs(1),
            _ => Duration::ZERO,
        }
    }

    fn _contact_damage(&self) -> u32 {
        match self {
            TowerType::SpikedWall => 5,
            _ => 0,
        }
    }

    fn _contact_damage_cooldown(&self) -> Duration {
        match self {
            TowerType::SpikedWall => Duration::from_secs(1),
            _ => Duration::ZERO,
        }
    }
}

impl Deref for Tower {
    type Target = TowerType;
    fn deref(&self) -> &Self::Target {
        &self.variant
    }
}

impl DerefMut for Tower {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.variant
    }
}
