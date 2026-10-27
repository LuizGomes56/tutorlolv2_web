//! Compact DOM payloads. Only the hovered element's documentation is rendered.
use crate::{components::image::ImageType, model::AbilityKind, utils::BINCODE_CONFIG};
use base64::{Engine as _, engine::general_purpose::STANDARD_NO_PAD};
use bincode::{Decode, Encode};
use std::fmt::Write;
use tutorlolv2::yew::render::{FormulaRenderer, MayFail};
use tutorlolv2::{AbilityId, CastId, ChampionId, Ctx, CtxVar, EntityId, ItemId, RuneId};
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Encode, Decode)]
pub enum Docs {
    Champion(ChampionId, AbilityId),
    /// Min and max abilities that are displayed together in a single cell.
    ChampionMerged(ChampionId, AbilityId, AbilityId),
    Item(ItemId),
    Rune(RuneId),
    ChampionGlobal(ChampionId),
    ItemGlobal(ItemId),
    RuneGlobal(RuneId),
    Builtin(Builtin, DocView),
}

#[derive(Clone, Copy, Debug, PartialEq, Encode, Decode)]
pub enum Builtin {
    Tower,
    BasicAttack,
    CriticalStrike,
    Onhit,
    Ignite,
}

#[derive(Clone, Copy, Debug, PartialEq, Encode, Decode)]
pub enum DocView {
    Global,
    Function,
}

// The enum is the wire schema; JavaScript treats the encoded attributes as opaque.
// These ephemeral payloads are produced and consumed by the same Wasm build.
impl Docs {
    // ChampionMerged encodes the Docs tag, ChampionId, and two AbilityId/AbilityName
    // pairs: six tags/IDs. Each fits within a varint u32 (at most five bytes).
    // Revisit this bound if a variant gains additional or variable-length fields.
    const MAX_ENCODED_BYTES: usize = 6 * 5;

    pub fn global(id: impl CastId) -> Self {
        match id.entity() {
            EntityId::Champion(id) => Self::ChampionGlobal(id),
            EntityId::Item(id) => Self::ItemGlobal(id),
            EntityId::Rune(id) => Self::RuneGlobal(id),
        }
    }

    pub fn encode(self) -> String {
        let mut bytes = [0_u8; Self::MAX_ENCODED_BYTES];

        let length = bincode::encode_into_slice(self, &mut bytes, BINCODE_CONFIG)
            .expect("Docs payload exceeded its documented encoded-size bound");

        STANDARD_NO_PAD.encode(&bytes[..length])
    }

    fn render(self, html: &mut String) -> MayFail {
        let body = match self {
            Self::ChampionMerged(champion, min, max) => {
                Self::Champion(champion, min).render(html)?;
                return Self::Champion(champion, max).render(html);
            }
            Self::Champion(champion, ability) => {
                if !champion.abilities().iter().any(|meta| meta.kind == ability) {
                    return Err("Unknown ability for champion".into());
                }

                champion.render_fn(ability)
            }
            Self::Item(item) => item.render_fn(),
            Self::Rune(rune) => rune.render_fn(),
            Self::ChampionGlobal(id) => id.render_global(),
            Self::ItemGlobal(id) => id.render_global(),
            Self::RuneGlobal(id) => id.render_global(),
            Self::Builtin(Builtin::Tower, DocView::Function) => Ok(FormulaRenderer::tower_fn()),
            Self::Builtin(Builtin::Tower, DocView::Global) => Ok(FormulaRenderer::tower_global()),
            Self::Builtin(Builtin::Onhit, DocView::Function) => {
                Ok(FormulaRenderer::onhit_effect_fn())
            }
            Self::Builtin(Builtin::Onhit, DocView::Global) => {
                Ok(FormulaRenderer::onhit_effect_global())
            }
            Self::Builtin(Builtin::CriticalStrike, DocView::Function) => {
                Ok(FormulaRenderer::critical_strike_fn())
            }
            Self::Builtin(Builtin::CriticalStrike, DocView::Global) => {
                Ok(FormulaRenderer::critical_strike_global())
            }
            Self::Builtin(Builtin::BasicAttack, DocView::Function) => {
                Ok(FormulaRenderer::basic_attack_fn())
            }
            Self::Builtin(Builtin::BasicAttack, DocView::Global) => {
                Ok(FormulaRenderer::basic_attack_global())
            }
            Self::Builtin(Builtin::Ignite, _) => Ok(FormulaRenderer::ignite_fn()),
        }?;

        html.push_str("<pre>");
        html.push_str(&body);
        html.push_str("</pre>");
        Ok(())
    }
}

/// Shared mapping for icon headers and combo controls. Do not attach this to
/// Image itself: an image inside a damage cell must retain that cell's docs.
impl ImageType {
    pub fn docs(self) -> Option<String> {
        let docs = match self {
            ImageType::Ability(champion, AbilityKind::Normal(ability)) => {
                Docs::Champion(champion, ability)
            }
            ImageType::Ability(champion, AbilityKind::Alias(merge)) => {
                let abilities = champion.abilities();
                Docs::ChampionMerged(
                    champion,
                    abilities[merge.min as usize].kind,
                    abilities[merge.max as usize].kind,
                )
            }
            ImageType::Champion(id) | ImageType::Centered(id) => Docs::global(id),
            ImageType::Item(id) => Docs::global(id),
            ImageType::Rune(id) => Docs::global(id),
            ImageType::Tower => Docs::Builtin(Builtin::Tower, DocView::Global),
            ImageType::BasicAttack => Docs::Builtin(Builtin::BasicAttack, DocView::Global),
            ImageType::CritStrike => Docs::Builtin(Builtin::CriticalStrike, DocView::Global),
            ImageType::OnhitAttack => Docs::Builtin(Builtin::Onhit, DocView::Global),
            ImageType::Ignite => Docs::Builtin(Builtin::Ignite, DocView::Global),
            _ => return None,
        };
        Some(docs.encode())
    }
}

/// Each record is a u8 CtxVar followed by the four little-endian f32 bytes.
pub fn encode_idents(idents: &[CtxVar], ctx: &Ctx) -> Option<String> {
    if idents.is_empty() {
        return None;
    }

    let mut bytes = Vec::with_capacity(idents.len() * 5);

    for &ident in idents {
        bytes.push(ident as u8);
        bytes.extend_from_slice(&ctx[ident].to_le_bytes());
    }

    Some(STANDARD_NO_PAD.encode(bytes))
}

/// Numeric value shown in identifier tooltips, also used to filter visible stats.
pub fn ident_display_value(ident: CtxVar, value: f32) -> f32 {
    match ident {
        CtxVar::AttackSpeed | CtxVar::BaseAttackSpeed | CtxVar::BonusAttackSpeed => {
            (value * 100.0).round() / 100.0 + 0.0
        }
        // Adding zero normalizes rounded negative zero for display.
        _ => value.round() + 0.0,
    }
}

fn render(idents: &str, docs: &str) -> MayFail<String> {
    let mut html = String::new();

    if !idents.is_empty() {
        let bytes = STANDARD_NO_PAD.decode(idents)?;

        if bytes.len() % 5 != 0 {
            return Err("Incomplete identifier record".into());
        }

        html.push_str("<div class=\"flex flex-col\">");

        for record in bytes.as_chunks::<5>().0 {
            let ident = CtxVar::from_repr(record[0]).ok_or("Unknown context variable")?;
            let value = ident_display_value(ident, f32::from_le_bytes(record[1..].try_into()?));
            let name = ident.to_snake_case();

            write!(
                html,
                "<div class=\"whitespace-nowrap\"><span class=\"C12\">{}</span> = <span class=\"C8\">{value}</span></div>",
                name
            )?;
        }

        html.push_str("</div>");
    }

    if !docs.is_empty() {
        let bytes = STANDARD_NO_PAD.decode(docs)?;
        let (doc, consumed): (Docs, _) =
            bincode::decode_from_slice(&bytes, BINCODE_CONFIG.with_limit::<65536>())?;

        if consumed != bytes.len() {
            return Err("Trailing documentation data".into());
        }

        doc.render(&mut html)?;
    }

    Ok(html)
}

#[wasm_bindgen]
pub fn render_hover_docs(idents: &str, docs: &str) -> Result<String, JsValue> {
    render(idents, docs).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::{IntoEnumIterator, VariantArray};

    #[test]
    fn displayed_ident_value_matches_zero_filter_and_rounding() {
        for value in [0.0, 0.0000038146973, -0.000061035156, 0.49, -0.49] {
            assert_eq!(ident_display_value(CtxVar::BonusMagicResist, value), 0.0);
        }
        for (value, expected) in [(0.5, 1.0), (-0.5, -1.0), (778.69995, 779.0)] {
            assert_eq!(ident_display_value(CtxVar::AbilityPower, value), expected);
        }
        for ident in [
            CtxVar::AttackSpeed,
            CtxVar::BaseAttackSpeed,
            CtxVar::BonusAttackSpeed,
        ] {
            assert_eq!(ident_display_value(ident, 0.0001), 0.0);
            assert_eq!(ident_display_value(ident, 1.44575), 1.45);
            assert_eq!(ident_display_value(ident, -0.42209995), -0.42);
            assert_eq!(ident_display_value(ident, 0.0), 0.0);
        }
    }

    #[test]
    fn docs_variants_fit_buffer_and_base64_contains_only_encoded_bytes() {
        let check = |doc: Docs| {
            let expected = bincode::encode_to_vec(doc, BINCODE_CONFIG).unwrap();
            let bytes = STANDARD_NO_PAD.decode(doc.encode()).unwrap();
            assert_eq!(bytes, expected, "{doc:?}");

            let (decoded, consumed): (Docs, _) =
                bincode::decode_from_slice(&bytes, BINCODE_CONFIG).unwrap();
            assert_eq!(decoded, doc);
            assert_eq!(consumed, bytes.len(), "trailing bytes for {doc:?}");
        };

        for &champion in ChampionId::VARIANTS {
            check(Docs::ChampionGlobal(champion));
            for min in champion.abilities() {
                check(Docs::Champion(champion, min.kind));
                for max in champion.abilities() {
                    check(Docs::ChampionMerged(champion, min.kind, max.kind));
                }
            }
        }

        // Also cover ability names that are not used in current champion metadata.
        let constructors: [fn(tutorlolv2::AbilityName) -> AbilityId; 5] = [
            AbilityId::P,
            AbilityId::Q,
            AbilityId::W,
            AbilityId::E,
            AbilityId::R,
        ];
        for name in tutorlolv2::AbilityName::iter() {
            for constructor in constructors {
                let ability = constructor(name);
                check(Docs::ChampionMerged(
                    ChampionId::default(),
                    ability,
                    ability,
                ));
            }
        }

        for &item in ItemId::VARIANTS {
            check(Docs::Item(item));
            check(Docs::ItemGlobal(item));
        }
        for &rune in RuneId::VARIANTS {
            check(Docs::Rune(rune));
            check(Docs::RuneGlobal(rune));
        }
        for builtin in [
            Builtin::Tower,
            Builtin::BasicAttack,
            Builtin::CriticalStrike,
            Builtin::Onhit,
            Builtin::Ignite,
        ] {
            for view in [DocView::Global, DocView::Function] {
                check(Docs::Builtin(builtin, view));
            }
        }
    }

    #[test]
    fn merged_header_docs_render_both_real_abilities() {
        let champion = ChampionId::VARIANTS
            .iter()
            .copied()
            .find(|id| !id.merge_data().is_empty())
            .unwrap();

        let merge = champion.merge_data()[0];
        let payload = ImageType::Ability(champion, AbilityKind::Alias(merge))
            .docs()
            .unwrap();

        let abilities = champion.abilities();
        let min = champion
            .render_fn(abilities[merge.min as usize].kind)
            .unwrap();

        let max = champion
            .render_fn(abilities[merge.max as usize].kind)
            .unwrap();

        assert_eq!(
            render("", &payload).unwrap(),
            format!("<pre>{min}</pre><pre>{max}</pre>")
        );
    }

    #[test]
    fn icon_docs_use_global_entity_definitions() {
        let item = ItemId::default();
        let payload = ImageType::Item(item).docs().unwrap();

        assert_eq!(
            render("", &payload).unwrap(),
            format!("<pre>{}</pre>", item.render_global().unwrap())
        );

        assert!(ImageType::Level.docs().is_none());
    }

    #[test]
    fn global_docs_render_entity_definitions() {
        for (entity, expected) in [
            (
                Docs::global(ChampionId::default()),
                ChampionId::default().render_global().unwrap(),
            ),
            (
                Docs::global(ItemId::default()),
                ItemId::default().render_global().unwrap(),
            ),
            (
                Docs::global(RuneId::default()),
                RuneId::default().render_global().unwrap(),
            ),
        ] {
            let payload = entity.encode();

            assert_eq!(
                render("", &payload).unwrap(),
                format!("<pre>{expected}</pre>")
            );
        }
    }

    #[test]
    fn builtin_docs_preserve_function_vs_global() {
        for (builtin, function, global) in [
            (
                Builtin::Tower,
                FormulaRenderer::tower_fn(),
                FormulaRenderer::tower_global(),
            ),
            (
                Builtin::BasicAttack,
                FormulaRenderer::basic_attack_fn(),
                FormulaRenderer::basic_attack_global(),
            ),
            (
                Builtin::CriticalStrike,
                FormulaRenderer::critical_strike_fn(),
                FormulaRenderer::critical_strike_global(),
            ),
            (
                Builtin::Onhit,
                FormulaRenderer::onhit_effect_fn(),
                FormulaRenderer::onhit_effect_global(),
            ),
            (
                Builtin::Ignite,
                FormulaRenderer::ignite_fn(),
                FormulaRenderer::ignite_fn(),
            ),
        ] {
            for (view, expected) in [(DocView::Function, function), (DocView::Global, global)] {
                let payload = Docs::Builtin(builtin, view).encode();

                assert_eq!(
                    render("", &payload).unwrap(),
                    format!("<pre>{expected}</pre>"),
                    "{builtin:?} {view:?}"
                );
            }
        }
    }

    #[test]
    fn idents_round_trip_preserves_values_and_formats_display() {
        let ctx = Ctx {
            attack_damage: 123.25,
            ability_power: 778.69995,
            bonus_magic_resist: 0.0000038146973,
            attack_speed: 1.44575,
            base_attack_speed: 1.02365,
            bonus_attack_speed: 0.42209995,
            ..Default::default()
        };

        let encoded = encode_idents(
            &[
                CtxVar::AttackDamage,
                CtxVar::AbilityPower,
                CtxVar::BonusMagicResist,
                CtxVar::AttackSpeed,
                CtxVar::BaseAttackSpeed,
                CtxVar::BonusAttackSpeed,
            ],
            &ctx,
        )
        .unwrap();

        let bytes = STANDARD_NO_PAD.decode(&encoded).unwrap();
        assert_eq!(bytes.len(), 30);
        assert_eq!(f32::from_le_bytes(bytes[1..5].try_into().unwrap()), 123.25);

        let html = render(&encoded, "").unwrap();

        assert!(html.contains("attack_damage</span> = <span class=\"C8\">123</span>"));
        for (name, value) in [
            ("ability_power", "779"),
            ("bonus_magic_resist", "0"),
            ("attack_speed", "1.45"),
            ("base_attack_speed", "1.02"),
            ("bonus_attack_speed", "0.42"),
        ] {
            assert!(html.contains(&format!(
                "{name}</span> = <span class=\"C8\">{value}</span>"
            )));
        }
    }

    #[test]
    fn docs_render_the_selected_champion_item_and_rune() {
        let champion = ChampionId::default();
        let ability = champion.abilities()[0].kind;
        let item = ItemId::default();
        let rune = RuneId::default();

        for (doc, expected) in [
            (
                Docs::Champion(champion, ability),
                champion.render_fn(ability).unwrap(),
            ),
            (Docs::Item(item), item.render_fn().unwrap()),
            (Docs::Rune(rune), rune.render_fn().unwrap()),
        ] {
            let encoded = doc.encode();
            let html = render("", &encoded).unwrap();

            assert_eq!(html, format!("<pre>{expected}</pre>"));
            assert!(encoded.len() < html.len());
        }
    }

    #[test]
    fn empty_payloads_do_not_create_attributes_or_content() {
        assert_eq!(encode_idents(&[], &Ctx::default()), None);
        assert_eq!(render("", "").unwrap(), "");
    }

    #[test]
    fn malformed_payloads_are_errors() {
        assert!(render("!", "").is_err());
        assert!(render(&STANDARD_NO_PAD.encode([0, 1]), "").is_err());
        assert!(render(&STANDARD_NO_PAD.encode([255, 0, 0, 0, 0]), "").is_err());
        assert!(render("", "!").is_err());
        assert!(render("", &STANDARD_NO_PAD.encode([1, 255])).is_err());

        let mut valid = STANDARD_NO_PAD
            .decode(Docs::Item(ItemId::default()).encode())
            .unwrap();

        valid.push(0);

        assert!(render("", &STANDARD_NO_PAD.encode(valid)).is_err());
    }
}
