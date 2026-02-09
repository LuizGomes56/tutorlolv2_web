use crate::{
    components::image::{Image, ImageType},
    model::AbilityKind,
    utils::encode_offset,
};
use std::{ops::Range, rc::Rc};
use tutorlolv2_gen::{
    BASIC_ATTACK_OFFSET, CRITICAL_STRIKE_OFFSET, CastId, ChampionId, ItemId, ONHIT_EFFECT_OFFSET,
    RuneId, TypeMetadata,
};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct TableHeaderProps {
    #[prop_or(1)]
    pub skip: u8,
    pub champion_id: ChampionId,
    pub items_meta: Rc<[TypeMetadata<ItemId>]>,
    pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
}

#[component]
pub fn TableHeader(props: &TableHeaderProps) -> Html {
    let TableHeaderProps {
        skip,
        champion_id,
        ref items_meta,
        ref runes_meta,
    } = *props;

    let cache = champion_id.cache();
    let abilities_meta = cache.metadata;
    let abilities_to_merge = cache.merge_data;

    let abilities = {
        let meta_len = abilities_meta.len();
        let merge_len = abilities_to_merge.len();
        let mut result = Vec::with_capacity(meta_len - merge_len);

        let mut i = 0;
        'outer: while i < meta_len {
            let metadata = abilities_meta[i];
            let kind = metadata.kind;
            let mut ability_kind = AbilityKind::Normal(kind);
            let mut j = 0;
            'inner: while j < merge_len {
                let merge = abilities_to_merge[j];
                j += 1;
                if merge.maximum_damage == i as u8 {
                    i += 1;
                    continue 'outer;
                }
                if merge.minimum_damage == i as u8 {
                    ability_kind = AbilityKind::Alias(merge);
                    break 'inner;
                }
            }

            result.push((
                ImageType::Ability(champion_id, ability_kind),
                match ability_kind {
                    AbilityKind::Alias(merge_data) => {
                        vec![
                            champion_id.get_ability_formula(merge_data.minimum_damage as usize),
                            champion_id.get_ability_formula(merge_data.maximum_damage as usize),
                        ]
                    }
                    _ => vec![champion_id.get_ability_formula(i)],
                },
            ));
            i += 1;
        }

        result
    };

    let mut headers = Vec::with_capacity(
        skip as usize + 3 + abilities.len() + items_meta.len() + runes_meta.len(),
    );

    fn header<T: Copy + Into<ImageType> + CastId>(
        headers: &mut Vec<(ImageType, Vec<&'static Range<usize>>)>,
        slice: &Rc<[TypeMetadata<T>]>,
    ) {
        for metadata in slice.iter() {
            let kind = metadata.kind;
            headers.push((kind.into(), vec![kind.formula()]))
        }
    }

    headers.extend([
        (ImageType::BasicAttack, vec![&BASIC_ATTACK_OFFSET]),
        (ImageType::CritStrike, vec![&CRITICAL_STRIKE_OFFSET]),
        (ImageType::OnhitAttack, vec![&ONHIT_EFFECT_OFFSET]),
    ]);
    headers.extend(abilities);
    header(&mut headers, items_meta);
    header(&mut headers, runes_meta);

    html! {
        <thead>
            <tr>
                {for (0..skip).map(|_| html!(<th></th>))}
                {for headers.into_iter().enumerate().map(|(i, (src, offsets))| {
                    let data_offset = encode_offset(&offsets);
                    html! {
                        <th key={i} {data_offset}>
                            <Image
                                {src}
                                class={classes!(
                                    "w-fit", "justify-self-center"
                                )}
                            />
                        </th>
                    }
                })}
            </tr>
        </thead>
    }
}
