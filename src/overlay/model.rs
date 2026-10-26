use bincode::{Decode, Encode};
use strum::VariantArray;

#[derive(Clone, Copy, Debug, PartialEq, Eq, VariantArray)]
pub enum Widget {
    Damage,
    Recommendations,
    StackInsert,
    StackRemover,
    Stacks,
}

impl Widget {
    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Damage => "Damage table",
            Self::Recommendations => "Recommendations",
            Self::StackInsert => "Add stacks",
            Self::StackRemover => "Remove stacks",
            Self::Stacks => "Stack table",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub struct Panel {
    pub enabled: [bool; 5],
    pub keep_visible: [bool; 5],
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            enabled: [true; 5],
            keep_visible: [true, false, false, false, true],
        }
    }
}

impl Panel {
    pub const fn visible(&self, widget: Widget, focused: bool) -> bool {
        self.enabled[widget.index()] && (focused || self.keep_visible[widget.index()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommendations_and_editors_disappear_when_closed_by_default() {
        let panels = Panel::default();
        assert!(panels.visible(Widget::Recommendations, true));
        assert!(!panels.visible(Widget::Recommendations, false));
        assert!(!panels.visible(Widget::StackInsert, false));
        assert!(!panels.visible(Widget::StackRemover, false));
        assert!(panels.visible(Widget::Damage, false));
        assert!(panels.visible(Widget::Stacks, false));
    }

    #[test]
    fn users_can_keep_recommendations_or_hide_everything_when_closed() {
        let mut panels = Panel {
            keep_visible: [false, true, false, false, false],
            ..Default::default()
        };

        assert!(panels.visible(Widget::Recommendations, false));
        assert!(!panels.visible(Widget::Damage, false));
        panels.enabled[Widget::Recommendations.index()] = false;
        assert!(!panels.visible(Widget::Recommendations, true));
        assert!(!panels.visible(Widget::Recommendations, false));
        assert!(
            Widget::VARIANTS
                .iter()
                .all(|&widget| !panels.visible(widget, false))
        );
    }

    #[test]
    fn widget_preferences_survive_binary_roundtrip() {
        let panels = Panel {
            enabled: [true, false, true, false, true],
            keep_visible: [false, true, false, true, false],
        };
        let bytes = bincode::encode_to_vec(&panels, bincode::config::standard()).unwrap();
        let (loaded, read): (Panel, _) =
            bincode::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
        assert_eq!(loaded, panels);
        assert_eq!(read, bytes.len());
    }
}
