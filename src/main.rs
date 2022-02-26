use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevy Jam #1".to_string(),
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_cameras)
        .add_startup_system(setup)
        .run();
}

fn setup_cameras(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("Kenney Blue Letter Tiles/letter_A.png"),
        ..Default::default()
    });
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
        MovementRange(1),
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
    movement_range: MovementRange,
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
        .insert(UnitMove::new(movement_cost, movement_range))
        .insert(UnitVision::new(vision_range))
        .insert(CanDodge::new(dodge_cost))
        .insert(CanHeal::new(heal_cost, can_heal_amount))
        .insert(CanRepair::new(repair_cost, can_repair_amount));
}

#[derive(Component)]
struct Unit;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
struct GridPosition {
    x: i32,
    y: i32,
}

impl GridPosition {
    fn to_position(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }

    fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, 0.0)
    }

    fn move_dir(&self, direction: Direction) -> GridPosition {
        match direction {
            Direction::Up => GridPosition {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Down => GridPosition {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Left => GridPosition {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => GridPosition {
                x: self.x + 1,
                y: self.y,
            },
        }
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
    energy: Energy,
    max_energy: Energy,
}

impl UnitEnergy {
    fn new(max_energy: Energy) -> UnitEnergy {
        UnitEnergy {
            energy: max_energy,
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

#[derive(Clone, Copy, Debug)]
struct MovementRange(u8);

#[derive(Component)]
struct UnitMove {
    energy: Energy,
    movement_range: MovementRange,
}

impl UnitMove {
    fn new(energy: Energy, movement_range: MovementRange) -> Self {
        UnitMove {
            energy,
            movement_range,
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
    energy: Energy,
}

impl CanDodge {
    fn new(energy: Energy) -> CanDodge {
        CanDodge { energy }
    }
}

#[derive(Component)]
struct CanHeal {
    energy: Energy,
    amount: HitPoints,
}

impl CanHeal {
    fn new(energy: Energy, amount: HitPoints) -> Self {
        CanHeal { energy, amount }
    }
}

#[derive(Component)]
struct CanRepair {
    energy: Energy,
    amount: ArmorPoints,
}

impl CanRepair {
    fn new(energy: Energy, amount: ArmorPoints) -> Self {
        CanRepair { energy, amount }
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

#[derive(Component, Clone, Copy, Debug, PartialEq)]
struct MoveRequest {
    direction: Direction,
}

impl MoveRequest {
    fn new(direction: Direction) -> Self {
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
