use std::rc::Rc;
use tutorlolv2_gen::{AbilityId, ChampionId, ItemId, MergeData, RuneId, TypeMetadata};
use yew::prelude::*;

use crate::{
    components::image::Image,
    utils::{AbilityKind, ImageType},
};

#[derive(PartialEq, Properties)]
pub struct TableHeaderProps {
    pub champion_id: ChampionId,
    pub abilities_meta: Rc<[TypeMetadata<AbilityId>]>,
    pub abilities_to_merge: Rc<[MergeData]>,
    pub items_meta: Rc<[TypeMetadata<ItemId>]>,
    pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
}

#[component]
pub fn TableHeader(props: &TableHeaderProps) -> Html {
    let TableHeaderProps {
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

            result.push(html! {
                <th>
                    <Image src={ImageType::Ability(*champion_id, ability_kind)} />
                </th>
            });
            i += 1;
        }

        result
    };

    fn header<T: Copy + Into<ImageType>>(slice: &Rc<[TypeMetadata<T>]>) -> Html {
        slice
            .into_iter()
            .map(|metadata| {
                html! {
                    <th>
                        <Image src={metadata.kind.into()} />
                    </th>
                }
            })
            .collect::<Html>()
    }

    html! {
        <thead>
            <tr>
                <th><Image src={ImageType::BasicAttack} /></th>
                <th><Image src={ImageType::CritStrike} /></th>
                {abilities}
                {header(items_meta)}
                {header(runes_meta)}
            </tr>
        </thead>
    }
}
