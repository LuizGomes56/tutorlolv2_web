use crate::impl_reducible;
use bincode::{Decode, Encode};
use std::fmt::Display;
use tutorlolv2::{AbilityId, Ctx, Key, MergeData};

impl_reducible!(PlayerStats i32 {
    ability_power,
    armor,
    armor_penetration_flat,
    armor_penetration_percent,
    attack_damage,
    attack_speed,
    crit_chance,
    crit_damage,
    current_health,
    magic_penetration_flat,
    magic_penetration_percent,
    magic_resist,
    max_health,
    max_mana,
    current_mana
});

/// Enum that defines the team of some player.
/// - `CHAOS` is converted to [`Team::Red`],
/// - `ORDER` and any other variant matches [`Team::Blue`]
#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq)]
pub enum Team {
    Blue,
    Red,
}

#[derive(Clone, Copy, Debug, Decode, PartialEq)]
pub struct RangeDamage {
    pub minimum_damage: i32,
    pub maximum_damage: i32,
}

/// Struct holding the core champion stats of a player, where `T` is a
/// numeric type
#[derive(Clone, Copy, Debug, Decode, Default, Encode, PartialEq)]
pub struct BasicStats {
    pub armor: i32,
    pub attack_damage: i32,
    pub attack_speed: i32,
    pub magic_resist: i32,
    pub max_health: i32,
    pub max_mana: i32,
}

/// Holds the damage of the basic attack, critical strike damage, and onhits
#[derive(Clone, Copy, Debug, Decode, PartialEq)]
pub struct Attacks {
    /// Damage of the basic attack hit
    pub basic_attack: i32,
    /// Damage of the critical strike. For most champions, it represents a
    /// multipler of 1.75x the damage of the basic attack.
    pub critical_strike: i32,
    /// The onhit damage variant, containing the necessary information to
    /// display it as a range `{min} - {max}`
    pub onhit_damage: RangeDamage,
}

/// Holds the most simple stats that need to be used to calculate
/// the damage against this enemy. Note that it is similar to struct
/// [`BasicStats`], but without the `attack_damage` and `mana` fields,
/// which are fields that do not quantify any damage reduction the enemy
/// champion may take. Generic parameter `T` is supposed to be a numeric type
#[derive(Clone, Copy, Debug, Decode, Default, Encode, PartialEq)]
pub struct SimpleStats {
    pub armor: i32,
    pub max_health: i32,
    pub magic_resist: i32,
}

#[derive(Clone, Debug, Decode, PartialEq)]
pub struct Damages {
    pub attacks: Attacks,
    pub abilities: Box<[i32]>,
    pub items: Box<[i32]>,
    pub runes: Box<[i32]>,
    pub ctx: Ctx,
}

impl Damages {
    pub fn sum(&self) -> i32 {
        self.attacks.onhit_damage.minimum_damage
            + self.attacks.onhit_damage.maximum_damage
            + self.abilities.iter().sum::<i32>()
            + self.items.iter().sum::<i32>()
            + self.runes.iter().sum::<i32>()
    }
}

impl_reducible!(AbilityLevels u8 { q, w, e, r });

impl AbilityLevels {
    pub const ABILITIES: [Key; 4] = [Key::Q, Key::W, Key::E, Key::R];

    pub const ACTIONS: [fn(u8) -> AbilityLevelsAction; 4] = [
        AbilityLevelsAction::Q,
        AbilityLevelsAction::W,
        AbilityLevelsAction::E,
        AbilityLevelsAction::R,
    ];
}

impl_reducible!(Dragons u16 {
    ally_fire,
    ally_earth,
    ally_chemtech,
    enemy_earth
});

impl_reducible!(EnemyStats i32 {
    armor,
    current_health,
    magic_resist,
    max_health,
    missing_health
});

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AbilityKind {
    Alias(MergeData),
    Normal(AbilityId),
}

impl AbilityKind {
    pub const fn ability_id(&self) -> AbilityId {
        match self {
            AbilityKind::Alias(merge) => merge.alias,
            AbilityKind::Normal(ability_id) => *ability_id,
        }
    }
}

impl From<Key> for AbilityKind {
    fn from(value: Key) -> Self {
        AbilityId::from(value).into()
    }
}

impl From<AbilityId> for AbilityKind {
    fn from(value: AbilityId) -> Self {
        AbilityKind::Normal(value)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum StatType {
    AbilityPower,
    Armor,
    ArmorPenetrationFlat,
    ArmorPenetrationPercent,
    AttackDamage,
    AttackSpeed,
    CritChance,
    CritDamage,
    CurrentHealth,
    MagicPenetrationFlat,
    MagicPenetrationPercent,
    MagicResist,
    MaxMana,
    MaxHealth,
    CurrentMana,
}

impl Display for StatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatType::AbilityPower => write!(f, "Ability Power"),
            StatType::Armor => write!(f, "Armor"),
            StatType::ArmorPenetrationFlat => write!(f, "Armor Pen. Flat"),
            StatType::ArmorPenetrationPercent => write!(f, "Armor Pen. %"),
            StatType::AttackDamage => write!(f, "Attack Damage"),
            StatType::AttackSpeed => write!(f, "Attack Speed"),
            StatType::CritChance => write!(f, "Crit Chance"),
            StatType::CritDamage => write!(f, "Crit Damage"),
            StatType::CurrentHealth => write!(f, "Current Health"),
            StatType::MagicPenetrationFlat => write!(f, "Magic Pen. Flat"),
            StatType::MagicPenetrationPercent => write!(f, "Magic Pen. %"),
            StatType::MagicResist => write!(f, "Magic Resist"),
            StatType::MaxHealth => write!(f, "Max Health"),
            StatType::MaxMana => write!(f, "Max Mana"),
            StatType::CurrentMana => write!(f, "Current Mana"),
        }
    }
}
