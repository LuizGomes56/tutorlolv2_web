use crate::{
    calculator::FinalEnemy,
    components::image::Image,
    model::{Attacks, Damages, RangeDamage},
    overlay::Enemy,
    utils::{EnumCast, encode_offset},
};
use std::{collections::HashSet, ops::Range, rc::Rc};
use tutorlolv2_gen::{
    ABILITY_IDENTS, AbilityId, ChampionId, Ctx, DamageType, EvalIdent, ITEM_IDENTS, ItemId,
    MergeData, RUNE_CLOSURES, RUNE_IDENTS, RuneId, TypeMetadata,
};
use yew::prelude::*;

const CTX_VARIANTS: usize = size_of::<Ctx>() / size_of::<f32>();

pub struct Cell {
    damage_type: DamageType,
    min_dmg: i32,
    max_dmg: Option<i32>,
    offset_main: Option<u64>,
    offset_exc: Option<u64>,
    idents: &'static [EvalIdent],
    key: usize,
}

fn rdmg(metadata: &Rc<[TypeMetadata<RuneId>]>, damages: Box<[i32]>) -> Box<[Cell]> {
    let mlen = metadata.len();
    let dlen = damages.len();

    assert!(
        mlen == dlen,
        "Incompatible metadata vs box i32 len: [{mlen}m] [{dlen}d]"
    );

    let mut cells = Box::<[Cell]>::new_uninit_slice(mlen);
    let mut i = 0;

    while i < mlen {
        let TypeMetadata {
            kind, damage_type, ..
        } = metadata[i];
        let text = damages[i];
        let closure_range = &RUNE_CLOSURES[kind.index()];
        let offset_main = encode_offset(Some(closure_range.clone()));
        let idents = RUNE_IDENTS[kind.index()];
        cells[i].write(Cell {
            damage_type,
            min_dmg: text,
            offset_main,
            offset_exc: None,
            max_dmg: None,
            idents,
            key: i,
        });
        i += 1;
    }

    unsafe { cells.assume_init() }
}

impl Cell {
    pub fn display(self, ctx: Ctx) -> Html {
        let Self {
            damage_type,
            min_dmg,
            max_dmg,
            offset_main,
            offset_exc,
            idents,
            key,
        } = self;
        let ctx_array = unsafe { core::mem::transmute::<_, [f32; CTX_VARIANTS]>(ctx) };

        let class = classes!(
            "text-xs",
            "text-center",
            match damage_type {
                DamageType::Physical => "text-orange-500",
                DamageType::Magic => "text-sky-500",
                DamageType::Mixed => "text-indigo-500",
                DamageType::True => "text-white",
                DamageType::Adaptative => "text-purple-500",
                DamageType::Unknown => "text-emerald-500",
            }
        );

        let enc64 = |value: Option<u64>| value.as_ref().map(ToString::to_string);
        let data_idents = idents
            .into_iter()
            .map(|&ident| {
                let value = ctx_array[ident as usize];
                format!("[{ident}:{value}]")
            })
            .collect::<String>();

        let text = match max_dmg {
            Some(max) => format!("{min_dmg} - {max}"),
            None => min_dmg.to_string(),
        };

        html! {
            <td
                {key}
                {class}
                data-offset-main={enc64(offset_main)}
                data-offset-exc={enc64(offset_exc)}
                data-idents={data_idents}
            >
                {text}
            </td>
        }
    }
}

fn admg(
    metadata: &Rc<[TypeMetadata<AbilityId>]>,
    merge_data: &Rc<[MergeData]>,
    damages: Box<[i32]>,
) -> Box<[Cell]> {
    let mlen = metadata.len();
    let dlen = damages.len();
    let glen = merge_data.len();

    assert!(
        mlen == dlen,
        "[a] Lenght of damage cells must have the same amount of metadata"
    );

    struct ACell {
        min: i32,
        max: Option<i32>,
        min_i: u8,
        max_i: Option<u8>,
    }

    let len = dlen - glen;
    let mut data = Box::<[ACell]>::new_uninit_slice(len);
    let mut to_remove = Box::<[u8]>::new_uninit_slice(glen);

    let mut c = 0;
    let mut g = 0;
    let mut t = 0;
    while g < glen {
        let MergeData {
            minimum_damage,
            maximum_damage,
            alias,
        } = merge_data[g];

        let min_i = minimum_damage as usize;
        let max_i = maximum_damage as usize;
        let min = damages[min_i];
        let max = damages[max_i];

        if max != 0 && min != max {
            let ptr = data[min_i].as_mut_ptr();
            unsafe {
                (*ptr).max_i = Some(maximum_damage);
                (*ptr).max = Some(damages[max_i])
            }
        }

        to_remove[t].write(maximum_damage);
    }

    while c < len {
        unsafe {
            let acell_ptr = data[c].as_mut_ptr();
            let ptr_deref = &(*acell_ptr);
            (*acell_ptr).min = damages[ptr_deref.min_i as usize];
            c += 1;
        }
    }

    unsafe { data.assume_init() };

    todo!()
}

#[derive(PartialEq, Properties)]
pub struct TableBodyProps<T: PartialEq + 'static + DisplayDamage> {
    pub enemies: Rc<[T]>,
    pub ability_offsets: &'static [Range<usize>],
    pub abilities_meta: Rc<[TypeMetadata<AbilityId>]>,
    pub abilities_to_merge: Rc<[MergeData]>,
    pub items_meta: Rc<[TypeMetadata<ItemId>]>,
    pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
}

pub trait DisplayDamage {
    fn get_eval_ctx(&self) -> &Ctx;
    fn get_damages(&self) -> &Damages;
    fn get_champion_id(&self) -> ChampionId;
}

impl DisplayDamage for FinalEnemy {
    fn get_damages(&self) -> &Damages {
        &self.damages
    }
    fn get_champion_id(&self) -> ChampionId {
        self.champion_id
    }
    fn get_eval_ctx(&self) -> &Ctx {
        &self.eval_ctx
    }
}

impl DisplayDamage for Enemy {
    fn get_damages(&self) -> &Damages {
        &self.damages
    }
    fn get_champion_id(&self) -> ChampionId {
        self.champion_id
    }
    fn get_eval_ctx(&self) -> &Ctx {
        &self.eval_ctx
    }
}

#[component]
pub fn TableBody<T: PartialEq + 'static + DisplayDamage>(props: &TableBodyProps<T>) -> Html {
    let TableBodyProps {
        enemies,
        abilities_meta,
        abilities_to_merge,
        items_meta,
        runes_meta,
        ability_offsets,
    } = props;

    enemies
        .iter()
        .map(|enemy| {
            let damages = enemy.get_damages();
            let champion_id = enemy.get_champion_id();
            let eval_ctx = enemy.get_eval_ctx();
            let eval_meta = unsafe { core::mem::transmute::<_, &[f32; CTX_VARIANTS]>(eval_ctx) };

            let abilities = {
                let damages = &damages.abilities;
                let mut data = damages.iter().map(|&v| (v, None)).collect::<Vec<_>>();

                let mut to_remove = HashSet::with_capacity(abilities_to_merge.len());

                for merge in abilities_to_merge.iter() {
                    let min_idx = merge.minimum_damage as usize;
                    let max_idx = merge.maximum_damage as usize;

                    if let (Some(min_val), Some(max_val)) =
                        (damages.get(min_idx), damages.get(max_idx))
                    {
                        if *max_val != 0 && min_val != max_val {
                            data[min_idx].1 = Some(max_idx);
                        }
                        to_remove.insert(max_idx);
                    }
                }

                data.into_iter()
                    .enumerate()
                    .filter_map(|(i, (min_val, max_idx))| {
                        if to_remove.contains(&i) {
                            return None;
                        }

                        let text = match max_idx {
                            Some(max_i) => {
                                let max_val = damages[max_i];
                                format!("{min_val} - {max_val}")
                            }
                            None => min_val.to_string(),
                        };

                        Some((i, max_idx, text))
                    })
                    .collect::<Vec<_>>()
            };

            fn get_classes(damage_type: DamageType) -> Classes {
                classes!(
                    "text-xs",
                    "text-center",
                    match damage_type {
                        DamageType::Physical => "text-orange-500",
                        DamageType::Magic => "text-sky-500",
                        DamageType::Mixed => "text-indigo-500",
                        DamageType::True => "text-white",
                        DamageType::Adaptative => "text-purple-500",
                        DamageType::Unknown => "text-emerald-500",
                    }
                )
            }

            let rune_damages = damages
                .runes
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let damage_type = runes_meta[i].damage_type;
                    let offsets = RUNE_CLOSURES[runes_meta[i].kind.index()].clone();
                    let encoded = encode_offset(Some(offsets))
                        .as_ref()
                        .map(ToString::to_string);
                    html! {
                        <td key={i} data-offset-main={encoded} class={get_classes(damage_type)}>
                            {*item}
                        </td>
                    }
                })
                .collect::<Html>();

            let attacks = |value, damage_type| {
                html! {
                    <td class={get_classes(damage_type)}>
                        {value}
                    </td>
                }
            };

            let Attacks {
                basic_attack,
                critical_strike,
                onhit_damage:
                    RangeDamage {
                        minimum_damage: onhit_min,
                        maximum_damage: onhit_max,
                    },
            } = damages.attacks;

            let item_damages = items_meta
                .into_iter()
                .enumerate()
                .map(|(i, metadata)| {
                    let TypeMetadata {
                        kind, damage_type, ..
                    } = *metadata;
                    let minimum_damage = damages.items[i];
                    let maximum_damage = damages.items[i + 1];

                    let data = ITEM_IDENTS[kind.index()]
                        .into_iter()
                        .map(|ident| {
                            let value = eval_meta[*ident as usize];
                            format!("{ident}: {value}, ")
                        })
                        .collect::<String>();

                    html! {
                        <td
                            key={i}
                            data-eval={data}
                            class={get_classes(damage_type)}
                        >
                            {minimum_damage}{(
                                maximum_damage != 0
                                    && minimum_damage
                                    != maximum_damage
                                ).then_some(
                                html!(<>{" - "}{maximum_damage}</>)
                            )}
                        </td>
                    }
                })
                .collect::<Html>();

            html! {
                <tr>
                    <td>
                        <Image src={champion_id.image_type()} />
                    </td>
                    {attacks(basic_attack, DamageType::Physical)}
                    {attacks(critical_strike, DamageType::Physical)}
                    <td class={get_classes(DamageType::Mixed)}>
                        {onhit_min}{(onhit_max != 0 && onhit_max != onhit_min).then_some(
                            html!(<>{" - "}{onhit_max}</>)
                        )}
                    </td>
                    {abilities.into_iter().map(|(i, j, damage)| {
                        let damage_type = abilities_meta[i].damage_type;
                        let main_offset = encode_offset(Some(ability_offsets[i].clone()))
                            .as_ref()
                            .map(ToString::to_string);
                        let exc_offset = j.and_then(|j| {
                            encode_offset(Some(ability_offsets[j].clone()))
                                .as_ref()
                                .map(ToString::to_string)
                        });

                        html! {
                            <td
                                key={i}
                                data-offset-main={main_offset}
                                data-offset-exc={exc_offset}
                                class={get_classes(damage_type)}
                            >
                                {damage}
                            </td>
                        }
                    }).collect::<Html>()}
                    {item_damages}
                    {rune_damages}
                </tr>
            }
        })
        .collect::<Html>()
}
