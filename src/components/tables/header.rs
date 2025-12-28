use std::rc::Rc;
use tutorlolv2_gen::{AbilityId, ChampionId, ItemId, MergeData, RuneId, TypeMetadata};
use yew::prelude::*;

use crate::{
    components::image::Image,
    utils::{AbilityKind, ImageType},
};

#[derive(PartialEq, Properties)]
pub struct TableHeaderProps {
    #[prop_or(1)]
    pub skip: u8,
    pub champion_id: ChampionId,
    pub abilities_meta: Rc<[TypeMetadata<AbilityId>]>,
    pub abilities_to_merge: Rc<[MergeData]>,
    pub items_meta: Rc<[TypeMetadata<ItemId>]>,
    pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
}

#[component]
pub fn TableHeader(props: &TableHeaderProps) -> Html {
    let TableHeaderProps {
        skip,
        champion_id,
        abilities_meta,
        abilities_to_merge,
        items_meta,
        runes_meta,
    } = props;

    let abilities = {
        let meta_len = abilities_meta.len();
        let merge_len = abilities_to_merge.len();
        let mut result = Vec::with_capacity(meta_len - merge_len);

        let mut i = 0;
        'outer: while i < meta_len {
            let metadata = abilities_meta[i];
            let mut ability_kind = AbilityKind::Normal(metadata.kind);
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

            result.push(html!(<Image src={ImageType::Ability(*champion_id, ability_kind)} />));
            i += 1;
        }

        result
    };

    let mut headers = Vec::with_capacity(
        *skip as usize + 2 + abilities.len() + items_meta.len() + runes_meta.len(),
    );

    fn header<T: Copy + Into<ImageType>>(headers: &mut Vec<Html>, slice: &Rc<[TypeMetadata<T>]>) {
        for metadata in slice.into_iter() {
            headers.push(html!(<Image src={metadata.kind.into()} />))
        }
    }

    (0..*skip).for_each(|_| headers.push(html!()));
    [
        ImageType::BasicAttack,
        ImageType::CritStrike,
        ImageType::OnhitAttack,
    ]
    .into_iter()
    .for_each(|image| headers.push(html!(<Image src={image} />)));
    headers.extend(abilities);
    header(&mut headers, items_meta);
    header(&mut headers, runes_meta);

    html! {
        <thead>
            <tr>
                {for headers.into_iter().enumerate().map(|(i, value)| {
                    html!(<th key={i} class={classes!(
                        "justify-items-center", "py-0.5"
                    )}>{value}</th>)
                })}
            </tr>
        </thead>
    }
}
