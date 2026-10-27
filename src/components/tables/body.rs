use crate::{
    hover::{self, Builtin, DocView, Docs},
    livegame::Enemy,
    utils::ClassCast,
};
use tutorlolv2::{
    ChampionId, Ctx, CtxVar, DamageType, EntityId, EntityKind, ItemId, RuneId, TypeMetadata,
    ValueId,
    model::{Attacks, Damages, OutputEnemy},
};
use yew::prelude::*;

pub trait Victim {
    fn max_health(&self) -> i32;
    fn champion_id(&self) -> ChampionId;
    fn damages(&self) -> &Damages;
}

impl Victim for OutputEnemy {
    fn champion_id(&self) -> ChampionId {
        self.champion_id
    }

    fn max_health(&self) -> i32 {
        self.current_stats.max_health as _
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
        self.current_stats.max_health as _
    }
}

pub struct Cell {
    damage_type: DamageType,
    min_dmg: i32,
    max_dmg: Option<i32>,
    idents: &'static [CtxVar],
    docs: Docs,
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
            Some(max) if max == min && min != 0 => Some(Html::from(min)),
            Some(0) | None if min != 0 => Some(Html::from(min)),
            _ => None,
        }
    }

    pub fn render(self, ctx: &Ctx) -> Html {
        let Self {
            damage_type,
            min_dmg,
            max_dmg,
            idents,
            docs,
            diff,
        } = self;

        let data_idents = hover::encode_idents(idents, ctx);
        let data_docs = docs.encode();

        let main_line = Self::render_range(min_dmg, max_dmg).unwrap_or_else(|| Html::from(min_dmg));

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
                {data_docs}
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

pub struct DamageRenderer<'a> {
    pub damages: &'a Damages,
    pub champion_id: ChampionId,
    pub items_meta: &'a [TypeMetadata<ItemId>],
    pub runes_meta: &'a [TypeMetadata<RuneId>],
    pub other: Option<&'a Damages>,
    pub cells: Vec<Cell>,
}

impl<'a> DamageRenderer<'a> {
    pub fn new(
        damages: &'a Damages,
        champion_id: ChampionId,
        items_meta: &'a [TypeMetadata<ItemId>],
        runes_meta: &'a [TypeMetadata<RuneId>],
        other: Option<&'a Damages>,
    ) -> Self {
        let merge_data = champion_id.merge_data();
        let abilities_meta = champion_id.abilities();

        let len = Self::BASE_CELLS + abilities_meta.len() - merge_data.len()
            + items_meta.len()
            + runes_meta.len();

        Self {
            damages,
            champion_id,
            items_meta,
            runes_meta,
            other,
            cells: Vec::<Cell>::with_capacity(len),
        }
    }

    pub const BASE_CELLS: usize = 3;

    pub fn ability_cell_index(&self) -> Vec<usize> {
        let Self { champion_id, .. } = *self;

        let merge_data = champion_id.merge_data();
        let abilities_meta = champion_id.abilities();

        let len = abilities_meta.len();

        let mut indexes = vec![usize::MAX; len];
        let mut max_iterator = merge_data.iter().map(|m| m.max as usize).peekable();

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

    pub fn attack_cells(&mut self) {
        let Self {
            ref mut cells,
            damages,
            other,
            ..
        } = *self;

        let Attacks {
            basic_attack,
            critical_strike,
            onhit_damage,
        } = damages.attacks;

        let data = [
            Cell {
                damage_type: DamageType::Physical,
                min_dmg: basic_attack,
                max_dmg: None,
                idents: &[CtxVar::AttackDamage, CtxVar::PhysicalMultiplier],
                docs: Docs::Builtin(Builtin::BasicAttack, DocView::Function),
                diff: other.map(|o| (basic_attack - o.attacks.basic_attack, None)),
            },
            Cell {
                damage_type: DamageType::Physical,
                min_dmg: critical_strike,
                max_dmg: None,
                idents: &[CtxVar::AttackDamage, CtxVar::CritDamage],
                docs: Docs::Builtin(Builtin::CriticalStrike, DocView::Function),
                diff: other.map(|o| (critical_strike - o.attacks.critical_strike, None)),
            },
            Cell {
                damage_type: DamageType::Mixed,
                min_dmg: onhit_damage.minimum_damage,
                max_dmg: Some(onhit_damage.maximum_damage),
                idents: &[],
                docs: Docs::Builtin(Builtin::Onhit, DocView::Function),
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

    pub fn abilities_damage(&mut self) {
        let Self {
            damages,
            champion_id,
            other,
            ..
        } = *self;

        let damages = &damages.abilities;

        let abilities_meta = champion_id.abilities();
        let merge_data = champion_id.merge_data();
        let ability_cell_index = self.ability_cell_index();
        let ability_idents = champion_id.identifiers();

        let meta_len = abilities_meta.len();

        let cells = &mut self.cells;

        debug_assert_eq!(damages.len(), meta_len);
        debug_assert_eq!(ability_idents.len(), meta_len);

        let mut md_end = 0;

        for i in 0..meta_len {
            if md_end < merge_data.len() && (merge_data[md_end].max as usize) == i {
                let md = &merge_data[md_end];
                let min_i = md.min as usize;

                let target = ability_cell_index[min_i];

                debug_assert!(target != usize::MAX);
                debug_assert!(target < cells.len(), "Found a max match without min");

                let max_dmg = damages[i];
                let mut_ref = &mut cells[target];

                mut_ref.max_dmg = Some(max_dmg);
                mut_ref.docs = Docs::ChampionMerged(
                    champion_id,
                    abilities_meta[min_i].kind,
                    abilities_meta[i].kind,
                );

                if let Some(o) = other
                    && let Some((_, max_diff)) = &mut mut_ref.diff
                {
                    *max_diff = Some(max_dmg - o.abilities[i]);
                }

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
                idents,
                docs: Docs::Champion(champion_id, meta.kind),
                diff: other.map(|o| (damages[i] - o.abilities[i], None)),
            });
        }
    }

    pub fn value_damage<T: ValueId>(&mut self, metadata: &[TypeMetadata<T>]) {
        let Self {
            ref mut cells,
            damages,
            other,
            ..
        } = *self;

        let damages = match T::KIND {
            EntityKind::Champion => unreachable!(),
            EntityKind::Item => &damages.items,
            EntityKind::Rune => &damages.runes,
        };

        let visible_len = metadata.len() << 1;

        debug_assert!(
            damages.len() == visible_len
                || (matches!(T::KIND, EntityKind::Item)
                    && damages.len() == visible_len + 2
                    && damages[visible_len..] == [0, 0])
        );

        for (k, meta) in metadata.iter().enumerate() {
            let min_i = k << 1;
            let min_dmg = damages[min_i];
            let max_dmg = damages[min_i + 1];
            let id = meta.kind;

            cells.push(Cell {
                damage_type: meta.damage_type,
                min_dmg,
                max_dmg: Some(max_dmg),
                idents: id.identifiers(),
                docs: match id.entity() {
                    EntityId::Item(item) => Docs::Item(item),
                    EntityId::Rune(rune) => Docs::Rune(rune),
                    EntityId::Champion(_) => unreachable!(),
                },
                diff: other.map(|o| {
                    let odmg = match T::KIND {
                        EntityKind::Champion => unreachable!(),
                        EntityKind::Item => &o.items,
                        EntityKind::Rune => &o.runes,
                    };

                    // A new purchase appends one item pair absent from the owned build.
                    debug_assert!(
                        odmg.len() == damages.len()
                            || (matches!(T::KIND, EntityKind::Item)
                                && odmg.len() + 2 == damages.len())
                    );

                    let diff_min = odmg.get(min_i).copied().unwrap_or(0);
                    let diff_max = odmg.get(min_i + 1).copied().unwrap_or(0);

                    (min_dmg - diff_min, Some(max_dmg - diff_max))
                }),
            });
        }
    }

    pub fn render_cells(self) -> Html {
        let Self { damages, cells, .. } = self;

        debug_assert_eq!(cells.len(), cells.capacity());

        cells
            .into_iter()
            .map(|cell| cell.render(&damages.ctx))
            .collect::<Html>()
    }
}

impl From<DamageRenderer<'_>> for Html {
    fn from(mut renderer: DamageRenderer<'_>) -> Self {
        let DamageRenderer {
            damages,
            other,
            items_meta,
            runes_meta,
            ..
        } = renderer;

        #[cfg(debug_assertions)]
        if let Some(other) = other {
            debug_assert!(
                other.items.len() == damages.items.len()
                    || other.items.len() + 2 == damages.items.len()
            );

            debug_assert_eq!(other.runes.len(), damages.runes.len());
        }

        renderer.attack_cells();
        renderer.abilities_damage();
        renderer.value_damage(items_meta);
        renderer.value_damage(runes_meta);
        renderer.render_cells()
    }
}

#[cfg(test)]
mod purchase_tests {
    use super::*;

    #[test]
    fn new_purchase_row_uses_zero_baseline_and_keeps_owned_diffs() {
        let damage = Damages {
            items: vec![30, 40, 50, 60].into_boxed_slice(),
            ..Default::default()
        };
        let baseline = Damages {
            items: vec![10, 20].into_boxed_slice(),
            ..Default::default()
        };
        let metadata = [
            ItemId::NashorsTooth.data().metadata,
            ItemId::BladeOfTheRuinedKing.data().metadata,
        ];

        let mut renderer = DamageRenderer::new(
            &damage,
            Default::default(),
            &metadata,
            Default::default(),
            Some(&baseline),
        );

        renderer.value_damage(&metadata);

        let cells = &renderer.cells;
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].diff, Some((20, Some(20))));
        assert_eq!(cells[1].diff, Some((50, Some(60))));
        assert!(matches!(
            cells[1].docs,
            Docs::Item(ItemId::BladeOfTheRuinedKing)
        ));
        assert_eq!(cells[1].min_dmg, 50);
        assert_eq!(cells[1].max_dmg, Some(60));
    }

    #[test]
    fn zero_candidate_pair_renders_for_owned_and_non_damaging_purchases() {
        for candidate in [ItemId::NashorsTooth, ItemId::VoidStaff] {
            let damage = Damages {
                items: vec![30, 40, 0, 0].into_boxed_slice(),
                ..Default::default()
            };
            let baseline = Damages {
                items: vec![10, 20].into_boxed_slice(),
                ..Default::default()
            };
            let metadata = [
                ItemId::NashorsTooth.data().metadata,
                candidate.data().metadata,
            ];

            let mut renderer = DamageRenderer::new(
                &damage,
                Default::default(),
                &metadata,
                Default::default(),
                Some(&baseline),
            );

            renderer.value_damage(&metadata);

            let cells = &mut renderer.cells;

            assert_eq!(cells.len(), 2);
            assert_eq!(cells[0].diff, Some((20, Some(20))));
            assert_eq!(cells[1].min_dmg, 0);
            assert_eq!(cells[1].max_dmg, Some(0));
            assert_eq!(cells[1].diff, Some((0, Some(0))));
            assert!(matches!(cells[1].docs, Docs::Item(item) if item == candidate));
        }
    }

    #[test]
    fn hidden_zero_candidate_preserves_owned_cells_and_rune_alignment() {
        let damage = Damages {
            items: vec![30, 40, 0, 0].into_boxed_slice(),
            runes: vec![8, 12].into_boxed_slice(),
            ..Default::default()
        };
        let baseline = Damages {
            items: vec![10, 20].into_boxed_slice(),
            runes: vec![3, 4].into_boxed_slice(),
            ..Default::default()
        };

        let items_meta = &[ItemId::NashorsTooth.data().metadata];
        let runes_meta = &[RuneId::Electrocute.data().metadata];

        let mut renderer = DamageRenderer::new(
            &damage,
            Default::default(),
            items_meta,
            runes_meta,
            Some(&baseline),
        );

        renderer.value_damage(items_meta);
        renderer.value_damage(runes_meta);

        let cells = &renderer.cells;

        assert_eq!(cells.len(), 2);
        assert!(matches!(cells[0].docs, Docs::Item(ItemId::NashorsTooth)));
        assert_eq!(cells[0].diff, Some((20, Some(20))));
        assert!(matches!(cells[1].docs, Docs::Rune(RuneId::Electrocute)));
        assert_eq!(cells[1].min_dmg, 8);
        assert_eq!(cells[1].diff, Some((5, Some(8))));
    }

    #[test]
    fn purchase_with_no_owned_damage_still_renders_its_pair() {
        let damage = Damages {
            items: vec![0, 70].into_boxed_slice(),
            ..Default::default()
        };
        let baseline = Damages::default();
        let metadata = [ItemId::NashorsTooth.data().metadata];

        let mut renderer = DamageRenderer::new(
            &damage,
            Default::default(),
            &metadata,
            Default::default(),
            Some(&baseline),
        );

        renderer.value_damage(&metadata);

        let cells = &renderer.cells;

        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].diff, Some((0, Some(70))));
        assert!(matches!(cells[0].docs, Docs::Item(ItemId::NashorsTooth)));
    }
}

#[cfg(test)]
mod ability_diff_tests {
    use super::*;

    #[test]
    fn ability_diffs_include_single_values_and_both_merged_endpoints() {
        let champion = ChampionId::Gnar;
        let len = champion.abilities().len();
        let merge = &champion.merge_data()[0];
        let single = (0..len)
            .find(|&i| {
                !champion
                    .merge_data()
                    .iter()
                    .any(|m| m.min as usize == i || m.max as usize == i)
            })
            .unwrap();
        let baseline = Damages {
            abilities: vec![100; len].into_boxed_slice(),
            ..Default::default()
        };
        let mut damage = baseline.clone();

        damage.abilities[merge.min as usize] = 125;
        damage.abilities[merge.max as usize] = 90;
        damage.abilities[single] = 130;

        let mut renderer = DamageRenderer::new(
            &damage,
            champion,
            Default::default(),
            Default::default(),
            Some(&baseline),
        );

        renderer.attack_cells();
        renderer.abilities_damage();

        let indexes = renderer.ability_cell_index();

        let cells = &renderer.cells;

        assert_eq!(
            cells[indexes[merge.min as usize]].diff,
            Some((25, Some(-10)))
        );
        assert_eq!(cells[indexes[single]].diff, Some((30, None)));

        let mut renderer2 = DamageRenderer::new(
            &damage,
            champion,
            Default::default(),
            Default::default(),
            None,
        );

        renderer2.attack_cells();
        renderer2.abilities_damage();

        let plain = &renderer2.cells;

        assert!(plain.iter().all(|cell| cell.diff.is_none()));
    }

    #[test]
    fn nonzero_differences_remain_visible_with_zero_or_absent_upper_bound() {
        for min in [-15, 25] {
            assert!(Cell::render_range(min, None).is_some());
            assert!(Cell::render_range(min, Some(0)).is_some());
            assert!(Cell::render_range(0, Some(min)).is_some());
        }
        assert!(Cell::render_range(0, None).is_none());
        assert!(Cell::render_range(0, Some(0)).is_none());
    }
}
