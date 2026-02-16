use crate::impl_reducible;
use bincode::{Decode, Encode};
use std::fmt::Display;
use tutorlolv2_gen::{AbilityId, AbilityName, Ctx, ItemId, MergeData, RuneId};

impl_reducible!(PlayerStats i32 {
    ability_power,
    armor,
    armor_penetration_flat,
    armor_penetration_percent,
    attack_damage,
    attack_range,
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
    pub max_health: i32,
    pub attack_damage: i32,
    pub magic_resist: i32,
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

impl_reducible!(AbilityLevels u8 { q, w, e, r });

impl AbilityLevels {
    pub const ABILITIES: [fn(AbilityName) -> AbilityId; 4] =
        [AbilityId::Q, AbilityId::W, AbilityId::E, AbilityId::R];
    pub const ACTIONS: [fn(u8) -> AbilityLevelsAction; 4] = [
        AbilityLevelsAction::Q,
        AbilityLevelsAction::W,
        AbilityLevelsAction::E,
        AbilityLevelsAction::R,
    ];
}

/// Wrapper around the type [`u32`], whose first [`Self::DISC_BITS`] are used to
/// identify the enum type of the current value, which is either [`ItemId`] or [`RuneId`],
/// and the remaining [`Self::VAL_BITS`] are used to store the actual number of stacks held
#[derive(Clone, Copy, Debug, Encode, PartialEq)]
#[repr(transparent)]
pub struct ValueException(u32);

impl ValueException {
    pub const DISC_BITS: u32 =
        Self::find_disc_bits(ItemId::VARIANTS as u32, RuneId::VARIANTS as u32);
    pub const VAL_BITS: u32 = 32 - Self::DISC_BITS;
    pub const VAL_MASK: u32 = (1u32 << Self::VAL_BITS) - 1;
    pub const DISC_MASK: u32 = !Self::VAL_MASK;
    pub const DISC_LOW_MASK: u32 = (1u32 << Self::DISC_BITS) - 1;

    /// Returns a u32 with the number of leading zeros of the maximum between `a` and `b`
    const fn find_disc_bits(a: u32, b: u32) -> u32 {
        u32::BITS - if a > b { a } else { b }.leading_zeros()
    }

    /// Returns how many stacks are stored. Note that it returns an [`u32`] but whose
    /// maximum value is [`Self::VAL_MASK`]
    pub const fn stacks(&self) -> u32 {
        self.0 & Self::VAL_MASK
    }

    /// Returns an [`u16`], which is large enough to represent both [`ItemId`] and [`RuneId`]
    /// enums. This value is taken from the first [`Self::DISC_BITS`] bits
    const fn enum_id(&self) -> u16 {
        ((self.0 >> Self::VAL_BITS) & ((1u32 << Self::DISC_BITS) - 1)) as u16
    }

    /// Returns if the current value is a [`RuneId`]
    pub const fn get_rune_id(&self) -> Option<RuneId> {
        RuneId::from_u8(self.enum_id() as u8)
    }

    /// Returns if the current value is an [`ItemId`]
    pub const fn get_item_id(&self) -> Option<ItemId> {
        ItemId::from_u16(self.enum_id())
    }

    /// If the value to be stored is greater than [`Self::VAL_MASK`],
    /// the value is truncated
    const fn truncate_value(v: u32) -> u32 {
        v & Self::VAL_MASK
    }

    /// Creates a new instance of [`Self`] from a [`RuneId`] and a number of stacks
    pub const fn pack_rune_id(r: RuneId, v: u32) -> Self {
        let disc = (r as u32) & Self::DISC_LOW_MASK;
        Self((disc << Self::VAL_BITS) | Self::truncate_value(v))
    }

    /// Creates a new instance of [`Self`] from an [`ItemId`] and a number of stacks
    pub const fn pack_item_id(i: ItemId, v: u32) -> Self {
        let disc = (i as u32) & Self::DISC_LOW_MASK;
        Self((disc << Self::VAL_BITS) | Self::truncate_value(v))
    }
}

impl_reducible!(Dragons u16 {
    ally_fire_dragons,
    ally_earth_dragons,
    ally_chemtech_dragons,
    enemy_earth_dragons
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

    pub const fn as_char(&self) -> char {
        self.ability_id().as_char()
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
    AttackRange,
    AttackSpeed,
    CritChance,
    CritDamage,
    CurrentHealth,
    MagicPenetrationFlat,
    MagicPenetrationPercent,
    MagicResist,
    MaxMana,
    MaxHealth,
    MissingHealth,
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
            StatType::AttackRange => write!(f, "Attack Range"),
            StatType::AttackSpeed => write!(f, "Attack Speed"),
            StatType::CritChance => write!(f, "Crit Chance"),
            StatType::CritDamage => write!(f, "Crit Damage"),
            StatType::CurrentHealth => write!(f, "Current Health"),
            StatType::MagicPenetrationFlat => write!(f, "Magic Pen. Flat"),
            StatType::MagicPenetrationPercent => write!(f, "Magic Pen. %"),
            StatType::MagicResist => write!(f, "Magic Resist"),
            StatType::MissingHealth => write!(f, "Missing Health"),
            StatType::MaxHealth => write!(f, "Max Health"),
            StatType::MaxMana => write!(f, "Max Mana"),
            StatType::CurrentMana => write!(f, "Current Mana"),
        }
    }
}
