use crate::{
    calculator::FinalEnemy,
    components::image::Image,
    model::{Attacks, Damages},
    utils::{EnumCast, VOID_MAIN_OFFSET, encode_offset, traits::ClassCast},
};
use std::{mem::MaybeUninit, ops::Range, rc::Rc};
use tutorlolv2_gen::{
    BASIC_ATTACK_FN_OFFSET, CRITICAL_STRIKE_FN_OFFSET, ChampionId, Ctx, CtxVar, DamageType, ItemId,
    MergeData, ONHIT_EFFECT_FN_OFFSET, RuneId, TypeMetadata,
};
use yew::prelude::*;

pub struct Cell {
    damage_type: DamageType,
    min_dmg: i32,
    max_dmg: Option<i32>,
    min_off: Range<usize>,
    max_off: Option<Range<usize>>,
    idents: &'static [CtxVar],
}

pub trait DamageCast {
    fn damages(&self) -> Rc<Damages>;
    fn champion_id(&self) -> ChampionId;
    fn ctx(&self) -> Ctx;
}

impl DamageCast for FinalEnemy {
    fn champion_id(&self) -> ChampionId {
        self.champion_id
    }
    fn ctx(&self) -> Ctx {
        self.eval_ctx
    }
    fn damages(&self) -> Rc<Damages> {
        self.damages.clone()
    }
}

type Meta<T> = Rc<[TypeMetadata<T>]>;

#[derive(PartialEq, Properties)]
pub struct TableBodyProps<T: PartialEq + 'static> {
    pub enemies: Rc<[T]>,
    pub champion_id: ChampionId,
    pub items_meta: Meta<ItemId>,
    pub runes_meta: Meta<RuneId>,
}

#[component]
pub fn TableBody<T: PartialEq + 'static + DamageCast>(props: &TableBodyProps<T>) -> Html {
    let TableBodyProps {
        enemies,
        champion_id,
        items_meta,
        runes_meta,
    } = props;

    let cache = champion_id.cache();
    let merge_data = cache.merge_data;
    let abilities_meta = cache.metadata;
    let ability_idents = champion_id.idents();
    let ability_idents_indexes = champion_id.ident_indexes();
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
        indexes.into_boxed_slice()
    };

    let len = 3 + abilities_meta.len() - merge_data.len() + items_meta.len() + runes_meta.len();

    enemies
        .iter()
        .map(|enemy| {
            let mut cells = Box::<[Cell]>::new_uninit_slice(len);

            unsafe {
                let damages = enemy.damages();
                let enemy_id = enemy.champion_id();
                let ctx = enemy.ctx();

                let Attacks {
                    basic_attack,
                    critical_strike,
                    onhit_damage,
                } = damages.attacks;

                [
                    Cell {
                        damage_type: DamageType::Physical,
                        min_dmg: basic_attack,
                        max_dmg: None,
                        min_off: BASIC_ATTACK_FN_OFFSET.clone(),
                        max_off: None,
                        idents: &[CtxVar::AttackDamage, CtxVar::PhysicalMultiplier],
                    },
                    Cell {
                        damage_type: DamageType::Physical,
                        min_dmg: critical_strike,
                        max_dmg: None,
                        min_off: CRITICAL_STRIKE_FN_OFFSET.clone(),
                        max_off: None,
                        idents: &[CtxVar::AttackDamage, CtxVar::CritDamage],
                    },
                    Cell {
                        damage_type: DamageType::Mixed,
                        min_dmg: onhit_damage.minimum_damage,
                        max_dmg: (onhit_damage.maximum_damage > 0)
                            .then_some(onhit_damage.maximum_damage),
                        min_off: ONHIT_EFFECT_FN_OFFSET.clone(),
                        max_off: None,
                        idents: &[],
                    },
                ]
                .into_iter()
                .enumerate()
                .for_each(|(i, cell)| {
                    cells[i] = MaybeUninit::new(cell);
                });

                debug_assert_eq!(items_meta.len(), damages.items.len() >> 1);
                debug_assert_eq!(runes_meta.len(), damages.runes.len());

                {
                    let abilities_dmg = &damages.abilities;
                    debug_assert_eq!(abilities_dmg.len(), abilities_meta.len());

                    let mut n = 0usize;

                    for i in 0..abilities_meta.len() {
                        let MergeData {
                            minimum_damage,
                            maximum_damage,
                            ..
                        } = merge_data[n];

                        let min_i = minimum_damage as usize;
                        let max_i = maximum_damage as usize;

                        if n < merge_data.len() && max_i == i {
                            let target = ability_cell_index[min_i];
                            debug_assert!(
                                target != usize::MAX,
                                "minimum_damage must generate a cell"
                            );

                            let ptr = cells[target].as_mut_ptr();
                            (*ptr).max_dmg = Some(abilities_dmg[i]);
                            (*ptr).max_off = Some(ability_closures[i].clone());

                            n += 1;
                            continue;
                        }

                        let cell_i = ability_cell_index[i];
                        debug_assert!(cell_i != usize::MAX);

                        let meta = &abilities_meta[i];

                        let id_range = ability_idents_indexes[i].clone();
                        let idents = &ability_idents[id_range];

                        let cell = Cell {
                            damage_type: meta.damage_type,
                            min_dmg: abilities_dmg[i],
                            max_dmg: None,
                            min_off: ability_closures[i].clone(),
                            max_off: None,
                            idents,
                        };

                        cells[cell_i] = MaybeUninit::new(cell);
                    }
                }

                let mut size = BASE_CELLS + abilities_meta.len() - merge_data.len();

                for (k, meta) in items_meta.iter().enumerate() {
                    let min_i = k << 1;
                    let min = damages.items[min_i];
                    let max = damages.items[min_i + 1];
                    let item_id = meta.kind;
                    let cell_i = size + k;

                    let cell = Cell {
                        damage_type: meta.damage_type,
                        min_dmg: min,
                        max_dmg: Some(max),
                        min_off: item_id.closure().clone(),
                        max_off: None,
                        idents: item_id.idents(),
                    };

                    cells[cell_i] = MaybeUninit::new(cell);
                }

                size += items_meta.len();

                for (k, meta) in runes_meta.iter().enumerate() {
                    let rune_id = meta.kind;
                    let cell_i = size + k;
                    let cell = Cell {
                        damage_type: meta.damage_type,
                        min_dmg: damages.runes[k],
                        max_dmg: None,
                        min_off: rune_id.closure().clone(),
                        max_off: None,
                        idents: rune_id.idents(),
                    };

                    cells[cell_i] = MaybeUninit::new(cell);
                }

                let table_cells = cells
                    .assume_init()
                    .into_iter()
                    .enumerate()
                    .map(|(key, cell)| {
                        let Cell {
                            damage_type,
                            min_dmg,
                            max_dmg,
                            min_off,
                            max_off,
                            idents,
                        } = cell;

                        let data_idents = idents
                            .iter()
                            .map(|&ident| {
                                let value = ctx[ident];
                                format!("{ident}:{value}")
                            })
                            .collect::<Vec<String>>()
                            .join("|");

                        let data_offset_main = encode_offset(&min_off);
                        let data_offset_exc = max_off.as_ref().map(encode_offset);

                        html! {
                            <td
                                {key}
                                {data_idents}
                                {data_offset_main}
                                {data_offset_exc}
                                class={damage_type.class()}>
                                {match max_dmg {
                                    Some(max_dmg) if max_dmg > 0 && max_dmg != min_dmg => html! {
                                        <>{min_dmg}{" - "}{max_dmg}</>
                                    },
                                    _ => html!(min_dmg)
                                }}
                            </td>
                        }
                    })
                    .collect::<Html>();

                let all_idents = CtxVar::ARRAY
                    .into_iter()
                    .skip(CtxVar::SKIP)
                    .map(|ident| {
                        let value = ctx[ident];
                        let name = &format!("{ident:?}")["Enemy".len()..];
                        format!("{name}:{value}")
                    })
                    .collect::<Vec<_>>()
                    .join("|");

                html! {
                    <tr>
                        <td data_offset_main={VOID_MAIN_OFFSET} data_idents={all_idents}>
                            <Image src={enemy_id.image_type()} />
                        </td>
                        {table_cells}
                    </tr>
                }
            }
        })
        .collect::<Html>()
}
