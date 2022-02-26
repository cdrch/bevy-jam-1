use bevy::{core::FixedTimestep, log::LogSettings, prelude::*};
use rand::Rng;
use std::ops::{Add, AddAssign};

const MAP_WIDTH: u32 = 5;
const MAP_HEIGHT: u32 = 5;
const MAP_SIZE: u32 = MAP_WIDTH * MAP_HEIGHT;
const TILE_WIDTH: f32 = 256.0;
const TILE_HEIGHT: f32 = 256.0;
const GAME_WIDTH: f32 = TILE_WIDTH * MAP_WIDTH as f32;
const GAME_HEIGHT: f32 = TILE_HEIGHT * MAP_HEIGHT as f32;
const REFERENCE_WIDTH: f32 = 1920.0;
const REFERENCE_HEIGHT: f32 = 1080.0;
const REFERENCE_ASPECT: f32 = REFERENCE_WIDTH / REFERENCE_HEIGHT;

struct ScreenSize {
    width: f32,
    height: f32,
}

impl ScreenSize {
    fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    fn aspect(&self) -> f32 {
        self.width / self.height
    }
}

fn main() {
    App::new()
        .insert_resource(LogSettings {
            level: bevy::log::Level::INFO,
            ..Default::default()
        })
        .insert_resource(WindowDescriptor {
            title: "Bevy Jam #1".to_string(),
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_cameras)
        .add_startup_system(setup)
        .add_system(position_translation)
        .add_system(size_scaling)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(order_random_movement),
        )
        .add_system(move_units)
        .run();
}

fn setup_cameras(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // commands.spawn_bundle(SpriteBundle {
    //     texture: asset_server.load("Kenney Blue Letter Tiles/letter_A.png"),
    //     ..Default::default()
    // });
    spawn_unit(
        commands,
        asset_server,
        &"Kenney Blue Letter Tiles/letter_B.png".to_string(),
        Faction::new("Player", 1),
        UnitController::Player,
        GridPosition { x: 0, y: 0 },
        HitPoints(100),
        ArmorPoints(0),
        WeaponStats::default(),
        WeaponStats::default(),
        WeaponStats::default(),
        Energy(128),
        Energy(4),
        Energy(8),
        VisionRange(8),
        Energy(32),
        HitPoints(1),
        Energy(128),
        ArmorPoints(0),
        Energy(0),
    )
}

fn spawn_unit(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    sprite_path: &String,
    faction: Faction,
    unit_controller: UnitController,
    grid_position: GridPosition,
    unit_hit_points: HitPoints,
    unit_armor_points: ArmorPoints,
    primary_weapon: WeaponStats,
    secondary_weapon: WeaponStats,
    tertiary_weapon: WeaponStats,
    unit_energy: Energy,
    energy_regeneration: Energy,
    movement_cost: Energy,
    vision_range: VisionRange,
    dodge_cost: Energy,
    can_heal_amount: HitPoints,
    heal_cost: Energy,
    can_repair_amount: ArmorPoints,
    repair_cost: Energy,
) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(sprite_path),
            transform: Transform::from_translation(grid_position.to_vec3()),
            ..Default::default()
        })
        .insert(Size::new(1.0, 1.0))
        .insert(Unit)
        .insert(unit_controller)
        .insert(grid_position)
        .insert(BelongsToFaction::new(&faction))
        .insert(UnitHitPoints::new(unit_hit_points))
        .insert(UnitArmorPoints::new(unit_armor_points))
        .insert(Weapon::new(primary_weapon))
        .insert(Weapon::new(secondary_weapon))
        .insert(Weapon::new(tertiary_weapon))
        .insert(UnitEnergy::new(unit_energy))
        .insert(EnergyRegeneration::new(energy_regeneration))
        .insert(UnitMove::new(movement_cost))
        .insert(UnitVision::new(vision_range))
        .insert(CanDodge::new(dodge_cost))
        .insert(CanHeal::new(heal_cost, can_heal_amount))
        .insert(CanRepair::new(repair_cost, can_repair_amount));
}

fn order_random_movement(
    mut commands: Commands,
    mut query: Query<(
        (&mut GridPosition, &mut UnitMove),
        (Without<MoveRequest>, Entity),
    )>,
) {
    let dir = random_direction();
    for ((grid_position, unit_move), (move_request, entity)) in &mut query.iter_mut() {
        commands
            .entity(entity)
            .insert(MoveRequest::with_direction(dir));
    }
}

fn try_move_unit(
    unit_position: &mut GridPosition,
    unit_move: &mut UnitMove,
    unit_energy: &mut UnitEnergy,
    direction: Direction,
) -> bool {
    // If the unit has enough energy to move, move the unit and
    // subtract the movement cost from the unit's energy.
    if unit_energy.current_energy.0 >= unit_move.energy_cost.0 {
        let old_pos = unit_position.clone();
        // Set the unit's GridPosition component to the new position
        *unit_position += direction.as_grid_position();
        if unit_position.x < 0
            || unit_position.x >= MAP_WIDTH as i32
            || unit_position.y < 0
            || unit_position.y >= MAP_HEIGHT as i32
        {
            *unit_position = old_pos;
            return true;
        }
        // If a movement occurred, subtract the movement cost from the unit's energy
        if old_pos != *unit_position {
            // Set the unit's UnitEnergy component to the new energy
            unit_energy.current_energy.0 -= unit_move.energy_cost.0;
            return true;
        }
    }
    false
}

fn move_units(
    mut commands: Commands,
    mut query: Query<(
        (&mut GridPosition, &mut UnitMove),
        (&mut UnitEnergy, (&mut MoveRequest, Entity)),
    )>,
) {
    // For each unit in the query, try to move it.
    for ((mut unit_position, mut unit_move), (mut unit_energy, (move_request, entity))) in
        query.iter_mut()
    {
        // Get the unit's current direction.
        let direction = move_request.direction;
        // Try to move the unit.
        let should_remove_order = try_move_unit(
            &mut unit_position,
            &mut unit_move,
            &mut unit_energy,
            direction,
        );
        if should_remove_order {
            // If the unit moved, remove the move order.
            // commands.remove_component::<MoveRequest>(move_request.entity);
            commands.entity(entity).remove::<MoveRequest>();
        }
    }
}

fn random_direction() -> Direction {
    let mut rng = rand::thread_rng();
    let random_number = rng.gen_range(0..4);
    match random_number {
        0 => Direction::Up,
        1 => Direction::Down,
        2 => Direction::Left,
        3 => Direction::Right,
        _ => panic!("Invalid random number"),
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Transform)>) {
    let window = windows.get_primary().unwrap();
    let aspect_ratio = window.width() as f32 / window.height() as f32;
    let scale = window.height() / REFERENCE_HEIGHT;
    for (sprite_size, mut transform) in q.iter_mut() {
        // println!("{:?}", sprite_size.width * aspect_ratio as f32);
        // println!("{:?}", sprite_size.height * aspect_ratio as f32);
        transform.scale = Vec3::new(
            sprite_size.width /*/ GAME_WIDTH as f32*/ * scale as f32,
            sprite_size.height /*/ GAME_HEIGHT as f32*/ * scale as f32,
            1.0,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&GridPosition, &mut Transform)>) {
    fn convert(pos: f32, window_bound: f32, game_bound: f32) -> f32 {
        let tile_size = TILE_WIDTH; //window_bound / game_bound;
                                    // pos * game_bound * window_bound - (window_bound / 2.) + (window_bound / game_bound / 2.)
        pos * tile_size
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, GAME_WIDTH as f32) * window.height()
                / REFERENCE_HEIGHT,
            convert(pos.y as f32, window.height() as f32, GAME_HEIGHT as f32) * window.height()
                / REFERENCE_HEIGHT,
            0.0,
        );
        info!("{:?}", transform.translation);
    }
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}
impl Size {
    fn new(width: f32, height: f32) -> Self {
        Size { width, height }
    }
    fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(Component)]
struct Unit;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
struct GridPosition {
    x: i32,
    y: i32,
}

impl Add for GridPosition {
    type Output = GridPosition;

    fn add(self, other: GridPosition) -> GridPosition {
        GridPosition {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for GridPosition {
    fn add_assign(&mut self, other: GridPosition) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl GridPosition {
    fn new(x: i32, y: i32) -> GridPosition {
        GridPosition { x, y }
    }

    fn to_position(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }

    fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, 0.0)
    }

    fn move_dir(&self, direction: Direction) -> (i32, i32) {
        match direction {
            Direction::Up => (self.x, self.y + 1),
            Direction::Down => (self.x, self.y - 1),
            Direction::Left => (self.x - 1, self.y),
            Direction::Right => (self.x + 1, self.y),
        }
    }

    fn set(&mut self, (x, y): (i32, i32)) {
        self.x = x;
        self.y = y;
    }

    fn get(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

#[derive(Component)]
struct Faction {
    name: String,
    id: i32,
}

impl Faction {
    fn new(name: &str, id: i32) -> Self {
        Faction {
            name: name.to_string(),
            id,
        }
    }
}

#[derive(Component)]
struct BelongsToFaction(i32);

impl BelongsToFaction {
    fn new_by_id(id: i32) -> Self {
        BelongsToFaction(id)
    }

    fn new(faction: &Faction) -> Self {
        BelongsToFaction(faction.id)
    }
}

#[derive(Clone, Copy, Debug)]
struct HitPoints(u8);

#[derive(Component)]
struct UnitHitPoints {
    current: HitPoints,
    max: HitPoints,
}

impl UnitHitPoints {
    fn new(max: HitPoints) -> Self {
        UnitHitPoints {
            current: HitPoints(max.0),
            max,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ArmorPoints(u8);

#[derive(Component)]
struct UnitArmorPoints {
    current: ArmorPoints,
    max: ArmorPoints,
}

impl UnitArmorPoints {
    fn new(max: ArmorPoints) -> Self {
        UnitArmorPoints {
            current: ArmorPoints(max.0),
            max,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Ammo(u8);

#[derive(Clone, Copy, Debug)]
struct Move(u8);

#[derive(Component)]
struct Weapon {
    stats: WeaponStats,
    current_ammo: Ammo,
}

impl Weapon {
    fn new(stats: WeaponStats) -> Self {
        Weapon {
            stats,
            current_ammo: stats.max_ammo,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct WeaponStats {
    armor_piercing: u8,
    accuracy: u8,
    damage: u8,
    agility_limit: u8,
    speed_limit: Move,
    max_ammo: Ammo,
    energy_cost: Energy,
}

impl Default for WeaponStats {
    fn default() -> Self {
        WeaponStats {
            armor_piercing: 0,
            accuracy: 64,
            damage: 4,
            agility_limit: 32,
            speed_limit: Move(255),
            max_ammo: Ammo(10),
            energy_cost: Energy(32),
        }
    }
}

impl WeaponStats {
    fn new(
        armor_piercing: u8,
        accuracy: u8,
        damage: u8,
        agility_limit: u8,
        speed_limit: Move,
        max_ammo: Ammo,
        energy_cost: Energy,
    ) -> Self {
        WeaponStats {
            armor_piercing,
            accuracy,
            damage,
            agility_limit,
            speed_limit,
            max_ammo,
            energy_cost,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Energy(i32);

#[derive(Component)]
struct UnitEnergy {
    current_energy: Energy,
    max_energy: Energy,
}

impl UnitEnergy {
    fn new(max_energy: Energy) -> UnitEnergy {
        UnitEnergy {
            current_energy: max_energy,
            max_energy,
        }
    }
}

#[derive(Component)]
struct EnergyRegeneration {
    energy_regeneration: Energy,
}

impl EnergyRegeneration {
    fn new(energy_regeneration: Energy) -> EnergyRegeneration {
        EnergyRegeneration {
            energy_regeneration,
        }
    }
}

#[derive(Component)]
struct UnitMove {
    energy_cost: Energy,
}

impl UnitMove {
    fn new(energy: Energy) -> Self {
        UnitMove {
            energy_cost: energy,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct VisionRange(u8);

#[derive(Component)]
struct UnitVision {
    vision_range: VisionRange,
}

impl UnitVision {
    fn new(vision_range: VisionRange) -> Self {
        UnitVision { vision_range }
    }
}

#[derive(Component)]
struct CanDodge {
    energy_cost: Energy,
}

impl CanDodge {
    fn new(energy: Energy) -> CanDodge {
        CanDodge {
            energy_cost: energy,
        }
    }
}

#[derive(Component)]
struct CanHeal {
    energy_cost: Energy,
    amount: HitPoints,
}

impl CanHeal {
    fn new(energy: Energy, amount: HitPoints) -> Self {
        CanHeal {
            energy_cost: energy,
            amount,
        }
    }
}

#[derive(Component)]
struct CanRepair {
    energy_cost: Energy,
    amount: ArmorPoints,
}

impl CanRepair {
    fn new(energy: Energy, amount: ArmorPoints) -> Self {
        CanRepair {
            energy_cost: energy,
            amount,
        }
    }
}

#[derive(Component)]
enum UnitController {
    Player,
    NPC,
}

impl Default for UnitController {
    fn default() -> Self {
        UnitController::NPC
    }
}

impl UnitController {
    fn is_player(&self) -> bool {
        match self {
            UnitController::Player => true,
            _ => false,
        }
    }

    fn is_npc(&self) -> bool {
        match self {
            UnitController::NPC => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }

    fn vector(&self) -> (i32, i32) {
        match self {
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::Up => (0, 1),
            Direction::Down => (0, -1),
        }
    }

    fn as_grid_position(&self) -> GridPosition {
        match self {
            Direction::Left => GridPosition::new(-1, 0),
            Direction::Right => GridPosition::new(1, 0),
            Direction::Up => GridPosition::new(0, 1),
            Direction::Down => GridPosition::new(0, -1),
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
struct MoveRequest {
    direction: Direction,
}

impl MoveRequest {
    fn with_direction(direction: Direction) -> Self {
        MoveRequest { direction }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum WeaponChoice {
    None,
    Primary,
    Secondary,
    Tertiary,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
struct AttackRequest {
    direction: Direction,
    weapon_choice: WeaponChoice,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
struct DodgeRequest;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
struct HealRequest {
    direction: Direction,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
struct RepairRequest {
    direction: Direction,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
struct WaitRequest;

#[derive(Component)]
struct Tile;
