//! Compact DOM payloads. Only the hovered element's documentation is rendered.
use crate::{components::image::ImageType, model::AbilityKind};
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
    pub fn global(id: impl CastId) -> Self {
        match id.entity() {
            EntityId::Champion(id) => Self::ChampionGlobal(id),
            EntityId::Item(id) => Self::ItemGlobal(id),
            EntityId::Rune(id) => Self::RuneGlobal(id),
        }
    }

    pub fn encode(self) -> String {
        let bytes = bincode::encode_to_vec(self, bincode::config::standard())
            .expect("Encoding documentation IDs into memory cannot fail");

        STANDARD_NO_PAD.encode(bytes)
    }

    fn render(self, html: &mut String) -> MayFail<()> {
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
            Self::Builtin(
                Builtin::BasicAttack | Builtin::CriticalStrike | Builtin::Onhit | Builtin::Ignite,
                view,
            ) => {
                match view {
                    // Placeholder: basic attack, critical strike, on-hit, and ignite
                    // functions are not implemented in the Formula builder yet.
                    DocView::Function => Ok(FormulaRenderer::tower_fn()),
                    // Placeholder: these globals are not implemented in the Formula
                    // builder yet; retain their distinct Builtin IDs for replacement.
                    DocView::Global => Ok(FormulaRenderer::tower_global()),
                }
            }
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

fn render(idents: &str, docs: &str) -> MayFail<String> {
    let mut html = String::new();

    if !idents.is_empty() {
        let bytes = STANDARD_NO_PAD.decode(idents)?;

        if bytes.len() % 5 != 0 {
            return Err("Incomplete identifier record".into());
        }

        html.push_str("<div class=\"flex flex-col\">");

        for record in bytes.chunks_exact(5) {
            let ident = CtxVar::from_repr(record[0]).ok_or("Unknown context variable")?;
            let value = f32::from_le_bytes(record[1..].try_into()?);

            write!(
                html,
                "<div class=\"whitespace-nowrap\"><span class=\"C12\">{}</span> = <span class=\"C8\">{value}</span></div>",
                ident.to_snake_case()
            )?;
        }

        html.push_str("</div>");
    }

    if !docs.is_empty() {
        let bytes = STANDARD_NO_PAD.decode(docs)?;
        let (doc, consumed): (Docs, _) =
            bincode::decode_from_slice(&bytes, bincode::config::standard().with_limit::<65536>())?;

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
    use strum::VariantArray;

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
    fn builtin_placeholders_preserve_function_vs_global() {
        for builtin in [
            Builtin::Tower,
            Builtin::BasicAttack,
            Builtin::CriticalStrike,
            Builtin::Onhit,
            Builtin::Ignite,
        ] {
            for (view, expected) in [
                (DocView::Function, FormulaRenderer::tower_fn()),
                (DocView::Global, FormulaRenderer::tower_global()),
            ] {
                let payload = Docs::Builtin(builtin, view).encode();
                assert_eq!(
                    render("", &payload).unwrap(),
                    format!("<pre>{expected}</pre>")
                );
            }
        }
    }

    #[test]
    fn idents_round_trip_names_and_float_values() {
        let mut ctx = Ctx::default();
        ctx.attack_damage = 123.25;
        ctx.physical_multiplier = -0.5;
        let encoded =
            encode_idents(&[CtxVar::AttackDamage, CtxVar::PhysicalMultiplier], &ctx).unwrap();
        assert_eq!(STANDARD_NO_PAD.decode(&encoded).unwrap().len(), 10);
        let html = render(&encoded, "").unwrap();
        assert!(html.contains("attack_damage</span> = <span class=\"C8\">123.25"));
        assert!(html.contains("physical_multiplier</span> = <span class=\"C8\">-0.5"));
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
