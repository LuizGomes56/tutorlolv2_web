use crate::{
    calculator::FinalEnemy,
    livegame::Enemy,
    model::{Attacks, Damages},
    utils::{ClassCast, encode_offset},
};
use std::{fmt::Write, ops::Range};
use tutorlolv2_gen::{
    BASIC_ATTACK_FN_OFFSET, CRITICAL_STRIKE_FN_OFFSET, ChampionId, CtxVar, DamageType, ItemId,
    ONHIT_EFFECT_FN_OFFSET, RuneId, TypeMetadata,
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
}

impl Damages {
    pub fn to_html(
        &self,
        champion_id: ChampionId,
        items_meta: &[TypeMetadata<ItemId>],
        runes_meta: &[TypeMetadata<RuneId>],
    ) -> Html {
        let merge_data = champion_id.merge_data();
        let abilities_meta = champion_id.abilities();
        let ability_idents = champion_id.idents();
        let ability_closures = champion_id.closures();

        const BASE_CELLS: usize = 3;

        let ability_cell_index = {
            let len = abilities_meta.len();

            let mut indexes = vec![usize::MAX; len];
            let mut max_iterator = merge_data
                .iter()
                .map(|m| m.maximum_damage as usize)
                .peekable();

            let mut pos = BASE_CELLS;
            (0..len).for_each(|i| match max_iterator.peek() {
                Some(&max) if max == i => {
                    max_iterator.next();
                }
                _ => {
                    indexes[i] = pos;
                    pos += 1;
                }
            });

            debug_assert_eq!(pos, BASE_CELLS + len - merge_data.len());
            indexes
        };

        let len = BASE_CELLS + abilities_meta.len() - merge_data.len()
            + items_meta.len()
            + runes_meta.len();

        let mut cells = Vec::<Cell>::with_capacity(len);

        let ctx = self.ctx;

        let Attacks {
            basic_attack,
            critical_strike,
            onhit_damage,
        } = self.attacks;

        debug_assert_eq!(items_meta.len(), self.items.len() >> 1);
        debug_assert_eq!(runes_meta.len(), self.runes.len());
        debug_assert_eq!(self.abilities.len(), abilities_meta.len());
        debug_assert_eq!(ability_idents.len(), abilities_meta.len());
        debug_assert_eq!(ability_closures.len(), abilities_meta.len());

        cells.extend([
            Cell {
                damage_type: DamageType::Physical,
                min_dmg: basic_attack,
                max_dmg: None,
                offsets: (&BASIC_ATTACK_FN_OFFSET, None),
                idents: &[CtxVar::AttackDamage, CtxVar::PhysicalMultiplier],
            },
            Cell {
                damage_type: DamageType::Physical,
                min_dmg: critical_strike,
                max_dmg: None,
                offsets: (&CRITICAL_STRIKE_FN_OFFSET, None),
                idents: &[CtxVar::AttackDamage, CtxVar::CritDamage],
            },
            Cell {
                damage_type: DamageType::Mixed,
                min_dmg: onhit_damage.minimum_damage,
                max_dmg: (onhit_damage.maximum_damage > 0).then_some(onhit_damage.maximum_damage),
                offsets: (&ONHIT_EFFECT_FN_OFFSET, None),
                idents: &[],
            },
        ]);

        let abilities_dmg = &self.abilities;
        let mut md_end = 0usize;

        for i in 0..abilities_meta.len() {
            if md_end < merge_data.len() && (merge_data[md_end].maximum_damage as usize) == i {
                let md = &merge_data[md_end];
                let min_i = md.minimum_damage as usize;

                let target = ability_cell_index[min_i];
                debug_assert!(target != usize::MAX);
                debug_assert!(target < cells.len(), "Found a max match without min");

                cells[target].max_dmg = Some(abilities_dmg[i]);
                cells[target].offsets.1 = Some(&ability_closures[i]);

                md_end += 1;
                continue;
            }

            let cell_i = ability_cell_index[i];
            debug_assert!(cell_i != usize::MAX);
            debug_assert_eq!(cell_i, cells.len());

            let meta = &abilities_meta[i];

            let idents = &ability_idents[i];

            cells.push(Cell {
                damage_type: meta.damage_type,
                min_dmg: abilities_dmg[i],
                max_dmg: None,
                offsets: (&ability_closures[i], None),
                idents,
            });
        }

        for (k, meta) in items_meta.iter().enumerate() {
            let min_i = k << 1;
            let min = self.items[min_i];
            let max = self.items[min_i + 1];

            let item_id = meta.kind;

            cells.push(Cell {
                damage_type: meta.damage_type,
                min_dmg: min,
                max_dmg: Some(max),
                offsets: (item_id.closure(), None),
                idents: item_id.idents(),
            });
        }

        for (k, meta) in runes_meta.iter().enumerate() {
            let rune_id = meta.kind;

            cells.push(Cell {
                damage_type: meta.damage_type,
                min_dmg: self.runes[k],
                max_dmg: None,
                offsets: (rune_id.closure(), None),
                idents: rune_id.idents(),
            });
        }

        debug_assert_eq!(cells.len(), len);

        cells
            .into_iter()
            .map(|cell| {
                let Cell {
                    damage_type,
                    min_dmg,
                    max_dmg,
                    offsets,
                    idents,
                } = cell;

                let mut data_idents = String::new();
                for (i, &ident) in idents.iter().enumerate() {
                    if i > 0 {
                        data_idents.push('|');
                    }
                    let value = ctx[ident];
                    let _ = write!(&mut data_idents, "{ident}:{value}");
                }

                let data_offset = {
                    let (a, b) = offsets;
                    match b {
                        Some(b) => encode_offset(&[a, b]),
                        None => encode_offset(core::slice::from_ref(&a)),
                    }
                };
                let dmg = match max_dmg {
                    Some(max) if max > 0 && max != min_dmg => html!(
                        <>{min_dmg}{" - "}{max}</>
                    ),
                    _ => html!(min_dmg),
                };

                html! {
                    <td
                        {data_idents}
                        {data_offset}
                        class={classes!("whitespace-nowrap", damage_type.class())}>
                        {dmg}
                    </td>
                }
            })
            .collect::<Html>()
    }
}
