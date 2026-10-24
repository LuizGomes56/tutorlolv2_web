---
version: alpha
name: Tutorlolv2
description: A dense, dark League of Legends theorycrafting calculator with visible champion loadouts.
colors:
  background: "#121212"
  surface: "#18181b"
  raised: "#202027"
  border: "#35353f"
  primary: "#67d9ed"
  gold: "#d7b775"
  text: "#f4f4f5"
  muted: "#a1a1aa"
  shop-background: "#12161e"
  shop-panel: "#191e29"
  shop-border: "#303744"
  shop-muted: "#b0b7c3"
  shop-gold: "#d4b779"
  shop-text: "#f0f2f7"
typography:
  sans:
    fontFamily: "ui-sans-serif, system-ui, sans-serif"
  mono:
    fontFamily: "ui-monospace, monospace"
rounded:
  sm: "4px"
  md: "14px"
spacing:
  control-gap: "8px"
  panel-padding: "16px"
components:
  loadout:
    backgroundColor: "{colors.background}"
    textColor: "{colors.text}"
    typography: "{typography.sans}"
    rounded: "{rounded.sm}"
  selector-dialog:
    backgroundColor: "{colors.shop-background}"
    textColor: "{colors.shop-text}"
    typography: "{typography.sans}"
    rounded: "{rounded.md}"
  item-card:
    backgroundColor: "{colors.shop-background}"
    textColor: "{colors.shop-text}"
  item-price:
    textColor: "{colors.gold}"
    typography: "{typography.sans}"
  build-help:
    textColor: "{colors.muted}"
---

# Tutorlolv2 design context

## Overview

Product tool for League players comparing damage and experimental builds. English UI; no specific geographic market. Keep the existing calculator's panel and table geometry. The user prefers a familiar in-game shop or a clean layout and rejected the earlier boxed item cards, repeated build cards, decorative header, and small names/prices. The visual reference is League's icon-led item shop: open item tiles, understated gold edges, one item inspector and one active inventory. Retain the six-slot minimum while allowing more than six entries. Avoid decorative neon, promotional subtitles and dashboard cards around individual choices.

The maintained direction is a modern dark game inventory: layered charcoal surfaces, subtle gradients, soft selected-item lighting, clear target grouping and restrained motion. Public visual references reviewed: [Figma Community dark RPG system and author previews](https://forum.figma.com/showcase-your-work-14/dark-rpg-game-ui-system-special-for-figma-community-52684), [Figma-built Dark RPG Inventory kit](https://canisius-ui.com/project-inventory). These guide layering and interaction states; the application retains its own layout, game imagery and typeface.

## Colors

Runtime CSS is canonical (ownership model B). Existing Tailwind `std` aliases in `public/tailwind.css` map to zinc. `public/main.css` defines scoped `--loadout-*` tokens consumed by the shared inventory and selector components: background→`--loadout-bg`, surface→`--loadout-surface`, raised→`--loadout-raised`, border→`--loadout-border`, primary→`--loadout-accent`, gold→`--loadout-gold`, text→`--loadout-text`, muted→`--loadout-muted`. This file mirrors those values. Gold means cost; cyan means active editing/selection. Rune paths retain distinct named, colored identities, never color alone.

`public/selectors.css` owns modal `--shop-*` tokens. The `shop-*` frontmatter colors map directly to the same CSS suffixes, except `shop-background`→`--shop-bg`. Shop controls use gold selection and muted blue-grey surfaces; panel loadouts retain the existing zinc/cyan styling.

## Typography

Preserve the app's system sans for names and controls. Catalog item names, rune names and prices are 16px, including narrow screens. The selected item uses a 20px title and 18px price on desktop. Prices and inventory costs use sans-serif with tabular numerals; never use monospace for prices. Reuse `/stats/gold.svg` (the bundled coin-stack artwork), at 18px next to prices. Secondary UI text is generally 14px. Item and rune names use their source spelling and remain visible; long compound names have optional balanced breaks without changing their text. No font download or layout shift.

## Layout

Preserve player/center/enemy layout and side-panel widths. Equipment uses six columns and wraps extra entries without imposing a domain limit. Desktop item dialogs use a compact stat-icon rail, open tile catalog and single inspector/inventory. Portrait buttons select the champion receiving items. Rune paths use a labeled rail and the same open choices. On narrow screens filters become horizontal and the item inspector/inventory forms a bottom dock. Landscape uses side-by-side catalog and inspector; short portrait permits workspace scrolling. Every scroll region has a named owner and a bounded height.

Desktop stat filters use one column with 16px horizontal padding and generous separation around the Stats heading. Catalog art is 48px (56px previously); the responsive tile minimum is 104px, producing seven items per row at a 1292px viewport and eight at a 1600px viewport. The current player has a full-width control above a separately labeled enemy group; never mix player and enemy controls in one row.

## Elevation & Depth

Layer charcoal and blue-grey tones, fine borders, subtle gradients and one restrained modal shadow. The native modal backdrop dims and blurs the calculator. Use cyan to identify the current player and gold for item selection and prices. Motion serves feedback: a 220ms dialog entrance, 160ms control/hover transitions and 280ms new-inventory-slot feedback. No perpetual animation. Reduced-motion disables animation, transforms and transitions.

## Shapes

Item artwork uses square slots with 7px corners; dialog controls use soft corners between 8px and 10px. Rune icons use circular wells. Dialogs use 14px corners.

## Components

`inputs/loadout.rs` owns panel inventories. `inputs/selector_dialog.rs` owns modal lifecycle, close/Done and controlled search. `item_selector.rs` owns shared active inventory, target portrait tabs, item inspector, role recommendation actions and item filtering. `rune_selector.rs` owns path groupings. Only show recommendations when the dataset actually provides them. Reuse existing reducers, `Image`, and hover documentation.

Native dialogs own inert background, focus trapping, Escape and focus return. Opening focuses search. Clicking an item inspects it; Add item equips it, and double-click is a shortcut that adds exactly once. Rune clicks add directly. Inventory removal is immediate. Done closes. Search and filters are transient in-memory editing aids, not route state. All actions use native buttons, named labels, visible hover/focus, pressed states where applicable. Empty search offers reset. Existing builds and legacy runes remain editable. No synthetic six-item or rune-tree restrictions.

`inputs/item_clicks.rs` recognizes rapid pointer-click pairs within 600ms even without a native dblclick event. Native dblclick remains a fallback for OS-configured timing, with duplicate suppression. Changing target, query, filters, modal state or using Add item resets the pending pair. Keyboard activation selects only. Regression coverage includes separate click events, native double-click deduplication, target/item isolation and four-click sequences.

Global scrollbars remain visible. Reduced-motion disables added transitions. Test current build edits, target switching, recommendations, empty search, keyboard and responsive open dialogs. Stack/combo is outside this change.

## Do's and Don'ts

Use the game's existing imagery as the signature. Keep names readable and builds visible while browsing. Don't make essential actions hover-only, hide extra build entries, or change calculator math or the overall page layout during selector work.
