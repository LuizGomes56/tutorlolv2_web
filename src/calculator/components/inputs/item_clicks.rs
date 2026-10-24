use crate::calculator::page::TargetEntity;
use tutorlolv2::ItemId;

type Click = (ItemId, TargetEntity, f64);

/// Recognize rapid click pairs without relying on a native dblclick event.
#[derive(Default)]
pub(super) struct ItemClicks {
    pending: Option<Click>,
    equipped: Option<Click>,
}

impl ItemClicks {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn click(&mut self, item: ItemId, target: TargetEntity, time: f64) -> bool {
        let pair = self.pending.take().is_some_and(|(previous, owner, at)| {
            previous == item && owner == target && (0.0..=600.0).contains(&(time - at))
        });
        if pair {
            self.equipped = Some((item, target, time));
        } else {
            self.pending = Some((item, target, time));
        }
        pair
    }

    /// Preserve OS-configured double-click timing without adding the same pair twice.
    pub fn native_double_click(&mut self, item: ItemId, target: TargetEntity, time: f64) -> bool {
        self.pending = None;
        let duplicate = self.equipped.is_some_and(|(previous, owner, at)| {
            previous == item && owner == target && (0.0..=80.0).contains(&(time - at))
        });
        if !duplicate {
            self.equipped = Some((item, target, time));
        }
        !duplicate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ITEM: ItemId = ItemId::InfinityEdge;
    const PLAYER: TargetEntity = TargetEntity::Player;

    #[test]
    fn two_separate_rapid_clicks_equip_once() {
        let mut clicks = ItemClicks::default();
        assert!(!clicks.click(ITEM, PLAYER, 100.0));
        assert!(clicks.click(ITEM, PLAYER, 300.0));
        assert!(!clicks.native_double_click(ITEM, PLAYER, 301.0));
    }

    #[test]
    fn native_double_click_supports_longer_os_timing() {
        let mut clicks = ItemClicks::default();
        assert!(!clicks.click(ITEM, PLAYER, 100.0));
        assert!(!clicks.click(ITEM, PLAYER, 900.0));
        assert!(clicks.native_double_click(ITEM, PLAYER, 901.0));
    }

    #[test]
    fn different_items_and_champions_cannot_form_a_pair() {
        let mut clicks = ItemClicks::default();
        assert!(!clicks.click(ITEM, PLAYER, 100.0));
        assert!(!clicks.click(ItemId::LongSword, PLAYER, 200.0));
        assert!(!clicks.click(ItemId::LongSword, TargetEntity::Enemy(0), 300.0));
    }

    #[test]
    fn four_clicks_equip_exactly_two_items() {
        let mut clicks = ItemClicks::default();
        let additions = [100.0, 200.0, 300.0, 400.0]
            .into_iter()
            .filter(|&at| clicks.click(ITEM, PLAYER, at))
            .count();
        assert_eq!(additions, 2);
    }

    #[test]
    fn reset_and_slow_clicks_do_not_equip() {
        let mut clicks = ItemClicks::default();
        assert!(!clicks.click(ITEM, PLAYER, 100.0));
        assert!(!clicks.click(ITEM, PLAYER, 800.0));
        clicks.reset();
        assert!(!clicks.click(ITEM, PLAYER, 900.0));
    }
}
