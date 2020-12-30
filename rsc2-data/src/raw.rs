use serde::Deserialize;

const fn one() -> f64 {
    1.0
}

#[derive(Debug, Deserialize)]
pub enum RawRace {
    Protoss,
    Zerg,
    Terran,
}

#[derive(Deserialize, Debug)]
pub enum RawAttributes {
    Armored,
    Mechanical,
    Massive,
    Structure,
    Light,
    Biological,
    Psionic,
    Heroic,
    Summoned,
}

#[derive(Debug, Deserialize)]
pub enum RawTarget {
    None,
    Point,
    Unit,
    PointOrUnit,
    Research {
        upgrade: usize,
        upgrade_name: String,
    },
    Build {
        produces: usize,
        produces_name: Option<String>,
    },
    BuildOnUnit {
        produces: usize,
        produces_name: String,
    },
    BuildInstant {
        produces: usize,
        produces_name: Option<String>,
    },
    Morph {
        produces: usize,
        produces_name: String,
    },
    Train {
        produces: usize,
        produces_name: Option<String>,
    },
    MorphPlace {
        produces: usize,
        produces_name: String,
    },
    TrainPlace {
        produces: usize,
        produces_name: String,
    },
}

#[derive(Deserialize, Debug)]
pub enum RawTargetType {
    Air,
    Ground,
    Any,
}

#[derive(Deserialize, Debug)]
pub struct RawBonus {
    pub against: String,
    pub damage: f64,
}

#[derive(Deserialize, Debug)]
pub struct RawWeapon {
    pub attacks: f64,
    pub bonuses: Vec<RawBonus>,
    pub cooldown: f64,
    pub damage_per_hit: f64,
    pub damage_splash: f64,
    pub range: f64,
    pub target_type: RawTargetType,
}

#[derive(Deserialize, Debug)]
pub struct Cost {
    pub minerals: usize,
    pub gas: usize,
    pub time: f64,
}
#[derive(Deserialize, Debug)]
pub struct UnitRawAbility {
    pub ability: usize,
}

#[derive(Deserialize, Debug)]
pub struct Energy {
    pub start_energy: usize,
    pub max_energy: f64,
}

#[derive(Deserialize, Debug)]
pub struct Cargo {
    pub cargo_size: usize,
    pub cargo_capacity: usize,
}

#[derive(Deserialize, Debug)]
pub struct RawUnit {
    pub id: usize,
    pub name: String,
    pub race: RawRace,

    #[serde(flatten)]
    pub cost: Cost,
    pub supply: f64,

    pub max_health: f64,
    pub size: usize,
    pub armor: f64,
    pub sight: f64,

    pub detection_range: Option<f64>,
    pub normal_mode: Option<usize>,

    #[serde(default = "one")]
    pub speed: f64,
    pub speed_creep_mul: f64,

    #[serde(flatten)]
    pub energy: Option<Energy>,
    #[serde(flatten)]
    pub cargo: Option<Cargo>,

    pub weapons: Vec<RawWeapon>,
    pub attributes: Vec<RawAttributes>,
    pub abilities: Vec<UnitRawAbility>,

    pub radius: Option<f64>,
    pub power_radius: Option<f64>,

    pub accepts_addon: bool,
    pub needs_power: bool,
    pub needs_creep: bool,
    pub needs_geyser: bool,
    pub is_structure: bool,
    pub is_addon: bool,
    pub is_worker: bool,
    pub is_townhall: bool,
}

#[derive(Deserialize, Debug)]
pub struct RawAbility {
    pub id: usize,
    pub name: String,
    pub cast_range: f64,
    pub energy_cost: usize,
    pub allow_minimap: bool,
    pub allow_autocast: bool,
    pub cooldown: usize,
    pub target: RawTarget,

    // always empty apparently
    #[serde(skip)]
    pub effect: Vec<()>,
    #[serde(skip)]
    pub buff: Vec<()>,
}

#[derive(Deserialize, Debug)]
pub struct RawUpgrade {
    pub id: usize,
    pub name: String,
    pub cost: Cost,
}

#[derive(Deserialize, Debug)]
pub struct RawData {
    #[serde(rename = "Ability")]
    pub ability: Vec<RawAbility>,
    #[serde(rename = "Unit")]
    pub unit: Vec<RawUnit>,
    #[serde(rename = "Upgrade")]
    pub upgrade: Vec<RawUpgrade>,
}
