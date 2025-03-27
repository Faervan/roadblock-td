use std::time::Duration;

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
    window::PrimaryWindow,
};

use crate::{
    Orientation, RngResource,
    animation::AnimationConfig,
    grid::{COLUMNS, Grid, GridPos, ROWS, TILE_SIZE, grid_to_world_coords, world_to_grid_coords},
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Enemy>()
            .register_type::<EnemyPath>()
            .register_type::<EnemyGoal>()
            .register_type::<EnemySpawn>()
            .add_systems(
                Startup,
                (
                    spawn_enemy_goal,
                    spawn_enemy_spawners.after(spawn_enemy_goal),
                ),
            )
            .add_systems(Update, (spawn_enemies, spawn_enemies_manual, move_enemies));
    }
}

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct Enemy {
    pub current: GridPos,
    pub goal: GridPos,
    variant: EnemyType,
    orientation: Orientation,
}

#[derive(Reflect)]
enum EnemyType {
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

    fn sprite_sheet(&self) -> &str {
        match self.variant {
            EnemyType::Skeleton => "sprites/enemies/BODY_skeleton.png",
        }
    }

    fn layout(&self, layouts: &mut Assets<TextureAtlasLayout>) -> TextureAtlas {
        match self.variant {
            EnemyType::Skeleton => TextureAtlas {
                layout: layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(64),
                    9,
                    4,
                    None,
                    None,
                )),
                index: self.sprite_indices().0,
            },
        }
    }

    fn offset(&self) -> Vec3 {
        match self.variant {
            EnemyType::Skeleton => Vec3::new(0., 10., 0.),
        }
    }

    fn scale(&self) -> Vec3 {
        match self.variant {
            EnemyType::Skeleton => Vec3::splat(0.6),
        }
    }

    fn animation_config(&self) -> AnimationConfig {
        match self.variant {
            EnemyType::Skeleton => {
                let (first, last) = self.sprite_indices();
                AnimationConfig::new(first, last, 10)
            }
        }
    }

    /// Returns (first_sprite_index, last_sprite_index)
    fn sprite_indices(&self) -> (usize, usize) {
        match self.variant {
            EnemyType::Skeleton => match self.orientation {
                Orientation::Up => (0, 8),
                Orientation::Down => (18, 26),
                Orientation::Left => (9, 17),
                Orientation::Right => (27, 35),
            },
        }
    }
}

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct EnemyPath {
    pub steps: Vec<GridPos>,
    next: Option<Vec3>,
}

impl EnemyPath {
    pub fn new(steps: Vec<GridPos>) -> Self {
        Self { steps, next: None }
    }
}

#[derive(Reflect, Component, Debug)]
#[reflect(Component)]
struct EnemySpawn {
    variant: EnemySpawnType,
    pos: GridPos,
    timer: Timer,
}

#[derive(Reflect, Debug)]
enum EnemySpawnType {
    RedTower,
}

impl EnemySpawn {
    fn new(variant: EnemySpawnType, pos: GridPos) -> Self {
        Self {
            variant,
            pos,
            timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating),
        }
    }

    /// Returns all the tiles that belong to the spawner, relative to the "origin tile"
    fn other_tiles(&self) -> Vec<GridPos> {
        match self.variant {
            EnemySpawnType::RedTower => {
                vec![
                    self.pos + GridPos::new(1, 0),
                    self.pos + GridPos::new(0, 1),
                    self.pos + GridPos::new(1, 1),
                ]
            }
        }
    }

    fn spawn_point(&self) -> Vec2 {
        grid_to_world_coords(self.pos)
            + match self.variant {
                EnemySpawnType::RedTower => Vec2::new(10., 0.),
            }
    }

    fn sprite(&self) -> &str {
        match self.variant {
            EnemySpawnType::RedTower => "sprites/spawners/red_spawner.png",
        }
    }

    fn offset(&self) -> Vec3 {
        match self.variant {
            EnemySpawnType::RedTower => Vec3::new(13., 15., 0.),
        }
    }

    fn scale(&self) -> Vec3 {
        match self.variant {
            EnemySpawnType::RedTower => Vec3::splat(0.8),
        }
    }
}

#[derive(Reflect, Component)]
#[reflect(Component)]
struct EnemyGoal;

fn spawn_enemy_spawners(
    mut commands: Commands,
    mut grid: ResMut<Grid>,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<RngResource>,
) {
    let mut origin_tiles = HashMap::new();
    let mut other_tiles = HashSet::new();
    let goal = grid.enemy_goal.iter().next().unwrap().0;

    while origin_tiles.len() != 5 {
        let [row, col] = [rng.0.isize(0..(ROWS - 1)), rng.0.isize(0..(COLUMNS - 1))];

        let spawner = EnemySpawn::new(EnemySpawnType::RedTower, GridPos::new(row, col));
        let other = spawner.other_tiles();

        if goal.distance_to(&spawner.pos) >= 20
            && !other.iter().any(|pos| other_tiles.contains(pos))
        {
            origin_tiles.insert(GridPos::new(row, col), spawner);
            other_tiles.extend(other);
        }
    }

    for (pos, spawner) in origin_tiles.into_iter() {
        let other = spawner.other_tiles();
        let entity = commands
            .spawn((
                Sprite::from_image(asset_server.load(spawner.sprite())),
                Transform {
                    translation: grid_to_world_coords(pos).extend(1.) + spawner.offset(),
                    scale: spawner.scale(),
                    ..Default::default()
                },
                spawner,
            ))
            .id();

        grid.enemy_spawn.insert(pos, entity);
        for tile in other.into_iter() {
            grid.enemy_spawn.insert(tile, entity);
        }
    }
}

fn spawn_enemy_goal(mut commands: Commands, mut grid: ResMut<Grid>) {
    let grid_pos = GridPos::new(ROWS / 2, COLUMNS - 1);
    let entity = commands
        .spawn((
            EnemyGoal,
            Sprite::from_color(Color::hsl(360., 1., 0.5), Vec2::splat(TILE_SIZE)),
            Transform {
                translation: grid_to_world_coords(grid_pos).extend(1.0),
                ..default()
            },
        ))
        .id();
    grid.enemy_goal.insert(grid_pos, entity);
}

fn spawn_enemies_manual(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    cam: Single<(&Camera, &GlobalTransform)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    grid: Res<Grid>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    if !mouse_input.just_pressed(MouseButton::Right) {
        return;
    }

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
                            image: asset_server.load(enemy.sprite_sheet()),
                            texture_atlas: Some(enemy.layout(&mut texture_atlas_layouts)),
                            ..Default::default()
                        },
                        Transform {
                            translation: grid_to_world_coords(grid_pos).extend(2.) + enemy.offset(),
                            scale: enemy.scale(),
                            ..default()
                        },
                        enemy.animation_config(),
                        enemy,
                    ));
                }
            }
        } else {
            warn!("Unable to get Cursor Position {:?}", world_pos.unwrap_err())
        }
    }
}

fn spawn_enemies(
    mut commands: Commands,
    grid: Res<Grid>,
    time: Res<Time>,
    mut spawners: Query<&mut EnemySpawn>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for mut spawner in &mut spawners {
        spawner.timer.tick(time.delta());
        if !spawner.timer.finished() {
            continue;
        }

        let enemy = Enemy::new(
            spawner.pos,
            *grid.enemy_goal.iter().next().unwrap().0,
            EnemyType::Skeleton,
        );

        commands.spawn((
            Sprite {
                image: asset_server.load(enemy.sprite_sheet()),
                texture_atlas: Some(enemy.layout(&mut texture_atlas_layouts)),
                ..Default::default()
            },
            Transform {
                translation: spawner.spawn_point().extend(2.) + enemy.offset(),
                scale: enemy.scale(),
                ..default()
            },
            enemy.animation_config(),
            enemy,
        ));
    }
}

fn move_enemies(
    mut query: Query<(
        &mut EnemyPath,
        &mut Enemy,
        &mut AnimationConfig,
        &mut Sprite,
        &mut Transform,
        Entity,
    )>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut path, mut enemy, mut animation, mut sprite, mut pos, entity) in &mut query {
        let next = match path.next {
            Some(tile) => tile,
            None => {
                if let Some(tile) = path.steps.pop() {
                    let orientation =
                        match (tile.row > enemy.current.row, tile.col > enemy.current.col) {
                            (true, false) => Orientation::Up,
                            (false, true) => Orientation::Right,
                            _ => match tile.row < enemy.current.row {
                                true => Orientation::Down,
                                false => Orientation::Left,
                            },
                        };
                    if orientation != enemy.orientation {
                        enemy.orientation = orientation;
                        *animation = enemy.animation_config();
                        if let Some(atlas) = &mut sprite.texture_atlas {
                            atlas.index = enemy.sprite_indices().0;
                        }
                    }
                    enemy.current = tile;
                    let next = grid_to_world_coords(tile).extend(2.) + enemy.offset();
                    path.next = Some(next);
                    next
                } else {
                    commands.entity(entity).despawn();
                    return;
                }
            }
        };
        let direction = next - pos.translation;
        pos.translation += direction.normalize() * time.delta_secs() * 150.;
        if pos.translation.distance(next) >= direction.length() {
            path.next = None;
        }
    }
}
