use attack::EnemyAttackPlugin;
use bevy::{input::common_conditions::input_just_pressed, prelude::*, window::PrimaryWindow};
use goal::EnemyGoalPlugin;
use movement::EnemyMovementPlugin;
pub use movement::PathChangedEvent;
use spawner::EnemySpawnerPlugin;

use crate::{
    Orientation,
    animation::AnimationConfig,
    grid::{Grid, GridPos, grid_to_world_coords, world_to_grid_coords},
};

mod attack;
mod goal;
mod movement;
mod spawner;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Enemy>()
            .add_plugins((
                EnemyMovementPlugin,
                EnemySpawnerPlugin,
                EnemyGoalPlugin,
                EnemyAttackPlugin,
            ))
            .add_systems(
                Update,
                spawn_enemies_manual.run_if(input_just_pressed(MouseButton::Right)),
            );
    }
}

#[derive(Reflect, Component, Deref, DerefMut)]
#[reflect(Component)]
pub struct Enemy {
    pub current: GridPos,
    pub goal: GridPos,
    #[deref]
    variant: EnemyType,
    orientation: Orientation,
}

#[derive(Reflect)]
pub enum EnemyType {
    Skeleton,
}

impl Enemy {
    fn new(current: GridPos, goal: GridPos, variant: EnemyType) -> Self {
        Self {
            current,
            goal,
            variant,
            orientation: Orientation::default(),
        }
    }

    fn walk_layout(&self, layouts: &mut Assets<TextureAtlasLayout>) -> TextureAtlas {
        match self.variant {
            EnemyType::Skeleton => TextureAtlas {
                layout: layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(64),
                    9,
                    4,
                    None,
                    None,
                )),
                index: self.walk_sprite_indices().0,
            },
        }
    }

    fn attack_layout(&self, layouts: &mut Assets<TextureAtlasLayout>) -> TextureAtlas {
        match self.variant {
            EnemyType::Skeleton => TextureAtlas {
                layout: layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(64),
                    6,
                    4,
                    None,
                    None,
                )),
                index: self.attack_sprite_indices().0,
            },
        }
    }

    fn walk_animation_config(&self) -> AnimationConfig {
        match self.variant {
            EnemyType::Skeleton => {
                let (first, last) = self.walk_sprite_indices();
                AnimationConfig::new(first, last, 10)
            }
        }
    }

    fn attack_animation_config(&self) -> AnimationConfig {
        match self.variant {
            EnemyType::Skeleton => {
                let (first, last) = self.attack_sprite_indices();
                AnimationConfig::new(first, last, 10)
            }
        }
    }

    /// Returns (first_sprite_index, last_sprite_index)
    fn walk_sprite_indices(&self) -> (usize, usize) {
        match self.variant {
            EnemyType::Skeleton => match self.orientation {
                Orientation::Up => (0, 8),
                Orientation::Down => (18, 26),
                Orientation::Left => (9, 17),
                Orientation::Right => (27, 35),
            },
        }
    }

    /// Returns (first_sprite_index, last_sprite_index)
    fn attack_sprite_indices(&self) -> (usize, usize) {
        match self.variant {
            EnemyType::Skeleton => match self.orientation {
                Orientation::Up => (0, 5),
                Orientation::Down => (12, 17),
                Orientation::Left => (6, 11),
                Orientation::Right => (18, 23),
            },
        }
    }
}

impl EnemyType {
    fn max_hp(&self) -> isize {
        match self {
            EnemyType::Skeleton => 25,
        }
    }

    fn damage(&self) -> isize {
        match self {
            EnemyType::Skeleton => 20,
        }
    }

    /// Cooldown between attacks in seconds
    fn attack_cooldown(&self) -> f32 {
        match self {
            EnemyType::Skeleton => 1.,
        }
    }

    fn travel_cost(&self, tower_hp: isize) -> usize {
        (tower_hp as f32 * self.attack_cooldown() / self.damage() as f32) as usize * 10
    }

    fn velocity(&self) -> f32 {
        match self {
            EnemyType::Skeleton => 150.,
        }
    }

    fn walk_sprites(&self) -> &str {
        match self {
            EnemyType::Skeleton => "sprites/enemies/BODY_skeleton_walk.png",
        }
    }

    fn attack_sprites(&self) -> &str {
        match self {
            EnemyType::Skeleton => "sprites/enemies/BODY_skeleton_attack.png",
        }
    }

    fn weapon_sprites(&self) -> &str {
        match self {
            EnemyType::Skeleton => "sprites/enemies/WEAPON_dagger.png",
        }
    }

    fn offset(&self) -> Vec3 {
        match self {
            EnemyType::Skeleton => Vec3::new(0., 10., 0.),
        }
    }

    fn scale(&self) -> Vec3 {
        match self {
            EnemyType::Skeleton => Vec3::splat(0.6),
        }
    }
}

/// Only for development purposes
fn spawn_enemies_manual(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    cam: Single<(&Camera, &GlobalTransform)>,
    grid: Res<Grid>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mouse_pos = window.cursor_position();

    if let Some(mouse_pos) = mouse_pos {
        let (camera, cam_transform) = *cam;

        let world_pos = camera.viewport_to_world_2d(cam_transform, mouse_pos);
        if let Ok(world_pos) = world_pos {
            if let Some(grid_pos) = world_to_grid_coords(world_pos) {
                if grid.is_free(&grid_pos) {
                    let enemy = Enemy::new(
                        grid_pos,
                        *grid.enemy_goal.iter().next().unwrap().0,
                        EnemyType::Skeleton,
                    );
                    commands.spawn((
                        Sprite {
                            image: asset_server.load(enemy.walk_sprites()),
                            texture_atlas: Some(enemy.walk_layout(&mut texture_atlas_layouts)),
                            ..Default::default()
                        },
                        Transform {
                            translation: grid_to_world_coords(grid_pos).extend(2.) + enemy.offset(),
                            scale: enemy.scale(),
                            ..default()
                        },
                        enemy.walk_animation_config(),
                        enemy,
                    ));
                }
            }
        } else {
            warn!("Unable to get Cursor Position {:?}", world_pos.unwrap_err())
        }
    }
}
