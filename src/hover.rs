//! Compact DOM payloads. Only the hovered element's documentation is rendered.
use base64::{Engine as _, engine::general_purpose::STANDARD_NO_PAD};
use bincode::{Decode, Encode};
use std::fmt::Write;
use tutorlolv2::yew::render::MayFail;
use tutorlolv2::{AbilityId, ChampionId, Ctx, CtxVar, ItemId, RuneId};
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Encode, Decode)]
pub enum Docs {
    Champion(ChampionId, AbilityId),
    Item(ItemId),
    Rune(RuneId),
}

// The enum is the wire schema; JavaScript treats the encoded attributes as opaque.
// These ephemeral payloads are produced and consumed by the same Wasm build.
impl Docs {
    fn render(self) -> MayFail<String> {
        match self {
            Self::Champion(champion, ability) => {
                if !champion.abilities().iter().any(|meta| meta.kind == ability) {
                    return Err("Unknown ability for champion".into());
                }

                champion.render_fn(ability)
            }
            Self::Item(item) => item.render_fn(),
            Self::Rune(rune) => rune.render_fn(),
        }
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

pub fn encode_docs(docs: &[Docs]) -> Option<String> {
    if docs.is_empty() {
        return None;
    }

    let bytes = bincode::encode_to_vec(docs, bincode::config::standard())
        .expect("Encoding documentation IDs into memory cannot fail");

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
        let (docs, consumed): (Vec<Docs>, _) =
            bincode::decode_from_slice(&bytes, bincode::config::standard().with_limit::<65536>())?;

        if consumed != bytes.len() {
            return Err("Trailing documentation data".into());
        }

        for doc in docs {
            html.push_str("<pre>");
            html.push_str(&doc.render()?);
            html.push_str("</pre>");
        }
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
        let encoded = encode_docs(&[
            Docs::Champion(champion, ability),
            Docs::Item(item),
            Docs::Rune(rune),
        ])
        .unwrap();
        let html = render("", &encoded).unwrap();
        assert_eq!(
            html,
            format!(
                "<pre>{}</pre><pre>{}</pre><pre>{}</pre>",
                champion.render_fn(ability).unwrap(),
                item.render_fn().unwrap(),
                rune.render_fn().unwrap()
            )
        );
        assert!(encoded.len() < html.len());
    }

    #[test]
    fn empty_payloads_do_not_create_attributes_or_content() {
        assert_eq!(encode_idents(&[], &Ctx::default()), None);
        assert_eq!(encode_docs(&[]), None);
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
            .decode(encode_docs(&[Docs::Item(ItemId::default())]).unwrap())
            .unwrap();
        valid.push(0);
        assert!(render("", &STANDARD_NO_PAD.encode(valid)).is_err());
    }
}
