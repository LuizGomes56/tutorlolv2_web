use crate::{
    calculator::FinalEnemy,
    livegame::Enemy,
    model::{Attacks, Damages},
    utils::{ClassCast, Print, encode_offset},
};
use core::{fmt::Write, ops::Range};
use tutorlolv2::AttackType;
use tutorlolv2_gen::{
    BASIC_ATTACK_FN_OFFSET, CRITICAL_STRIKE_FN_OFFSET, ChampionId, Ctx, CtxVar, DamageIndex,
    DamageType, EntityId, ItemId, ONHIT_EFFECT_FN_OFFSET, RuneId, TypeMetadata, ValueId,
};
use yew::prelude::*;

pub trait Victim {
    fn max_health(&self) -> i32;
    fn champion_id(&self) -> ChampionId;
    fn damages(&self) -> &Damages;
}

impl Victim for FinalEnemy {
    fn champion_id(&self) -> ChampionId {
        self.champion_id
    }

    fn max_health(&self) -> i32 {
        self.current_stats.max_health
    }

    fn damages(&self) -> &Damages {
        &self.damages
    }
}

impl Victim for Enemy {
    fn champion_id(&self) -> ChampionId {
        self.champion_id
    }

    fn damages(&self) -> &Damages {
        &self.damages
    }

    fn max_health(&self) -> i32 {
        self.current_stats.max_health
    }
}

pub struct Cell {
    damage_type: DamageType,
    min_dmg: i32,
    max_dmg: Option<i32>,
    offsets: (&'static Range<usize>, Option<&'static Range<usize>>),
    idents: &'static [CtxVar],
    diff: Option<(i32, Option<i32>)>,
}

impl Cell {
    fn render_value(value: i32) -> Html {
        match value < 0 {
            true => html!(<>{"("}{value}{")"}</>),
            false => html!(value),
        }
    }

    fn render_range(min: i32, max: Option<i32>) -> Option<Html> {
        const F: fn(i32) -> Html = Cell::render_value;

        match max {
            Some(max) if max != 0 && max > min => Some(html!(
                <>
                    {F(min)}
                    {" - "}
                    {F(max)}
                </>
            )),
            Some(max) if max != 0 && max < min => Some(html!(
                <>
                    {F(max)}
                    {" - "}
                    {F(min)}
                </>
            )),
            Some(max) if max == min => Some(Html::from(min)),
            None if min != 0 => Some(Html::from(min)),
            _ => None,
        }
    }

    pub fn render(self, ctx: &Ctx) -> Html {
        let Self {
            damage_type,
            min_dmg,
            max_dmg,
            offsets,
            idents,
            diff,
        } = self;

        let mut data_idents = String::new();
        for (i, &ident) in idents.iter().enumerate() {
            if i > 0 {
                data_idents.push('|');
            }
            let value = ctx[ident];
            let identifier = &ident.as_var()[4..];
            let _ = write!(&mut data_idents, "{identifier}:{value}");
        }

        let data_offset = {
            let (a, b) = offsets;
            match b {
                Some(b) => encode_offset(&[a, b]),
                None => encode_offset(core::slice::from_ref(&a)),
            }
        };

        let main_line = Self::render_range(min_dmg, max_dmg).unwrap_or(Html::from(min_dmg));

        let diff_line = diff.map(|(diff_min, diff_max)| {
            html! {
                <span class={classes!("text-std-400", "text-xs")}>
                    {Self::render_range(diff_min, diff_max).unwrap_or_default()}
                </span>
            }
        });

        html! {
            <td
                {data_idents}
                {data_offset}
                class={classes!("whitespace-nowrap", damage_type.class())}
            >
                <div class={classes!("flex", "flex-col", "items-center", "leading-tight")}>
                    <span>{main_line}</span>
                    {diff_line.unwrap_or_default()}
                </div>
            </td>
        }
    }
}

impl Damages {
    const BASE_CELLS: usize = 3;

    fn ability_cell_index(champion_id: ChampionId) -> Vec<usize> {
        let merge_data = champion_id.merge_data();
        let abilities_meta = champion_id.abilities();

        let len = abilities_meta.len();

        let mut indexes = vec![usize::MAX; len];
        let mut max_iterator = merge_data
            .iter()
            .map(|m| m.maximum_damage as usize)
            .peekable();

        let mut pos = Self::BASE_CELLS;
        (0..len).for_each(|i| match max_iterator.peek() {
            Some(&max) if max == i => {
                max_iterator.next();
            }
            _ => {
                indexes[i] = pos;
                pos += 1;
            }
        });

        debug_assert_eq!(pos, Self::BASE_CELLS + len - merge_data.len());
        indexes
    }

    fn attack_cells(&self, cells: &mut Vec<Cell>, other: Option<&Damages>) {
        let Attacks {
            basic_attack,
            critical_strike,
            onhit_damage,
        } = self.attacks;

        let data = [
            Cell {
                damage_type: DamageType::Physical,
                min_dmg: basic_attack,
                max_dmg: None,
                offsets: (&BASIC_ATTACK_FN_OFFSET, None),
                idents: &[CtxVar::AttackDamage, CtxVar::PhysicalMultiplier],
                diff: other.map(|o| (basic_attack - o.attacks.basic_attack, None)),
            },
            Cell {
                damage_type: DamageType::Physical,
                min_dmg: critical_strike,
                max_dmg: None,
                offsets: (&CRITICAL_STRIKE_FN_OFFSET, None),
                idents: &[CtxVar::AttackDamage, CtxVar::CritDamage],
                diff: other.map(|o| (critical_strike - o.attacks.critical_strike, None)),
            },
            Cell {
                damage_type: DamageType::Mixed,
                min_dmg: onhit_damage.minimum_damage,
                max_dmg: Some(onhit_damage.maximum_damage),
                offsets: (&ONHIT_EFFECT_FN_OFFSET, None),
                idents: &[],
                diff: other.map(|o| {
                    let other_onhit = &o.attacks.onhit_damage;
                    (
                        onhit_damage.minimum_damage - other_onhit.minimum_damage,
                        Some(onhit_damage.maximum_damage - other_onhit.maximum_damage),
                    )
                }),
            },
        ];

        cells.extend(data);
    }

    fn abilities_damage(
        &self,
        cells: &mut Vec<Cell>,
        champion_id: ChampionId,
        other: Option<&Damages>,
    ) {
        let damages = &self.abilities;

        let abilities_meta = champion_id.abilities();
        let merge_data = champion_id.merge_data();
        let ability_cell_index = Self::ability_cell_index(champion_id);
        let ability_idents = champion_id.identifiers();
        let ability_closures = champion_id.closures();

        let meta_len = abilities_meta.len();

        debug_assert_eq!(damages.len(), meta_len);
        debug_assert_eq!(ability_idents.len(), meta_len);
        debug_assert_eq!(ability_closures.len(), meta_len);

        let mut md_end = 0;

        for i in 0..meta_len {
            if md_end < merge_data.len() && (merge_data[md_end].maximum_damage as usize) == i {
                let md = &merge_data[md_end];
                let min_i = md.minimum_damage as usize;

                let target = ability_cell_index[min_i];
                debug_assert!(target != usize::MAX);
                debug_assert!(target < cells.len(), "Found a max match without min");

                let max_dmg = damages[i];
                let mut_ref = &mut cells[target];

                mut_ref.max_dmg = Some(max_dmg);
                mut_ref.offsets.1 = Some(&ability_closures[i]);

                if let Some(o) = other
                    && let Some((_, max_diff)) = &mut mut_ref.diff
                {
                    *max_diff = Some(max_dmg - o.abilities[i]);
                }

                mut_ref.offsets.1 = Some(&ability_closures[i]);
                md_end += 1;

                continue;
            }

            let cell_i = ability_cell_index[i];
            debug_assert!(cell_i != usize::MAX);
            debug_assert_eq!(cell_i, cells.len());

            let meta = &abilities_meta[i];
            let idents = ability_idents[i];

            cells.push(Cell {
                damage_type: meta.damage_type,
                min_dmg: damages[i],
                max_dmg: None,
                offsets: (&ability_closures[i], None),
                idents,
                diff: None,
            });
        }
    }

    fn value_damage<T: ValueId>(
        &self,
        cells: &mut Vec<Cell>,
        metadata: &[TypeMetadata<T>],
        attack_type: AttackType,
        other: Option<&Damages>,
    ) {
        let tag = T::default();
        let damages = match tag.entity() {
            EntityId::Champion(_) => {
                panic!("Can't use method value_damage with champions")
            }
            EntityId::Item(_) => &self.items,
            EntityId::Rune(_) => &self.runes,
        };

        debug_assert_eq!(metadata.len(), damages.len() >> 1);

        for (k, meta) in metadata.iter().enumerate() {
            let min_i = k << 1;
            let min_dmg = damages[min_i];
            let max_dmg = damages[min_i + 1];
            let id = meta.kind;

            cells.push(Cell {
                damage_type: meta.damage_type,
                min_dmg,
                max_dmg: Some(max_dmg),
                offsets: (
                    &id.functions()[attack_type as usize][DamageIndex::Min as usize],
                    None,
                ),
                idents: &id.identifiers()[attack_type as usize][DamageIndex::Min as usize],
                diff: other.map(|o| {
                    let diff_min = o.items[min_i];
                    let diff_max = o.items[min_i + 1];
                    (min_dmg - diff_min, Some(max_dmg - diff_max))
                }),
            });
        }
    }

    pub fn render_cells(&self, cells: Vec<Cell>) -> Html {
        debug_assert_eq!(cells.len(), cells.capacity());

        cells
            .into_iter()
            .map(|cell| cell.render(&self.ctx))
            .collect::<Html>()
    }

    pub fn make_cells(
        champion_id: ChampionId,
        items_meta: &[TypeMetadata<ItemId>],
        runes_meta: &[TypeMetadata<RuneId>],
    ) -> Vec<Cell> {
        let merge_data = champion_id.merge_data();
        let abilities_meta = champion_id.abilities();

        let len = Self::BASE_CELLS + abilities_meta.len() - merge_data.len()
            + items_meta.len()
            + runes_meta.len();

        Vec::<Cell>::with_capacity(len)
    }

    pub fn to_html(
        &self,
        champion_id: ChampionId,
        items_meta: &[TypeMetadata<ItemId>],
        runes_meta: &[TypeMetadata<RuneId>],
        other: Option<&Damages>,
    ) -> Html {
        let mut cells = Self::make_cells(champion_id, items_meta, runes_meta);

        if let Some(other) = other {
            debug_assert_eq!(other.items.len(), self.items.len());
            debug_assert_eq!(other.runes.len(), self.runes.len());
        }

        self.attack_cells(&mut cells, other);
        self.abilities_damage(&mut cells, champion_id, other);
        self.value_damage(&mut cells, items_meta, champion_id.attack_type(), other);
        self.value_damage(&mut cells, runes_meta, champion_id.attack_type(), other);
        self.render_cells(cells)
    }
}
