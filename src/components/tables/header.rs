use crate::{
    components::image::{Image, ImageType},
    model::AbilityKind,
    utils::encode_offset,
};
use std::{ops::Range, rc::Rc};
use tutorlolv2::{
    CastId, ChampionId, ItemId, RuneId, TypeMetadata,
    docs::{BASIC_ATTACK_OFFSET, CRITICAL_STRIKE_OFFSET, ONHIT_EFFECT_OFFSET},
};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct TableHeaderProps {
    #[prop_or(1)]
    pub skip: usize,
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

    let abilities_meta = champion_id.abilities();
    let abilities_to_merge = champion_id.merge_data();

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
                if merge.max == i as u8 {
                    i += 1;
                    continue 'outer;
                }
                if merge.min == i as u8 {
                    ability_kind = AbilityKind::Alias(merge);
                    break 'inner;
                }
            }

            result.push((
                ImageType::Ability(champion_id, ability_kind),
                champion_id.get_ability_formula(i),
            ));
            i += 1;
        }

        result
    };

    let mut headers =
        Vec::with_capacity(skip + 3 + abilities.len() + items_meta.len() + runes_meta.len());

    fn header<T: Copy + Into<ImageType> + CastId>(
        headers: &mut Vec<(ImageType, &'static Range<usize>)>,
        slice: &Rc<[TypeMetadata<T>]>,
    ) {
        for metadata in slice.iter() {
            let kind = metadata.kind;
            headers.push((kind.into(), kind.formula()))
        }
    }

    headers.extend([
        (ImageType::BasicAttack, &BASIC_ATTACK_OFFSET),
        (ImageType::CritStrike, &CRITICAL_STRIKE_OFFSET),
        (ImageType::OnhitAttack, &ONHIT_EFFECT_OFFSET),
    ]);
    headers.extend(abilities);
    header(&mut headers, items_meta);
    header(&mut headers, runes_meta);

    html! {
        <thead>
            <tr>
                {for (0..skip).map(|_| html!(<th></th>))}
                {for headers.into_iter().map(|(src, offsets)| {
                    let data_offset = encode_offset(&[offsets]);
                    html! {
                        <th {data_offset}>
                            <Image
                                {src}
                                class={classes!(
                                    "flex", "items-center",
                                    "justify-center",
                                    "place-self-center",
                                    "w-8", "h-8"
                                )}
                            />
                        </th>
                    }
                })}
            </tr>
        </thead>
    }
}
