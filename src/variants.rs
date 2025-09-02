use serde::{Deserialize, Serialize};
use std::{fmt::Error, str::FromStr};

// ========== CURSOR VARIANTS ==========

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum CursorVariant {
    #[default]
    Block,
    Beam,
    Underline,
    BlinkingBeam,
    BlinkingBlock,
    BlinkingUnderline,
}

impl std::str::FromStr for CursorVariant {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "beam" => Ok(Self::Beam),
            "block" => Ok(Self::Block),
            "underline" => Ok(Self::Underline),
            "blinking-beam" => Ok(Self::BlinkingBeam),
            "blinking-block" => Ok(Self::BlinkingBlock),
            "blinking-underline" => Ok(Self::BlinkingUnderline),
            _ => Err(Error),
        }
    }
}

impl CursorVariant {
    pub const ALL: &'static [Self] = &[
        Self::Beam,
        Self::Block,
        Self::Underline,
        Self::BlinkingBeam,
        Self::BlinkingBlock,
        Self::BlinkingUnderline,
    ];
    pub const NAME: &'static str = "cursor";

    pub fn value(&self) -> &'static str {
        match self {
            Self::Beam => "beam",
            Self::Block => "block",
            Self::Underline => "underline",
            Self::BlinkingBeam => "blinking-beam",
            Self::BlinkingBlock => "blinking-block",
            Self::BlinkingUnderline => "blinking-underline",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Beam => "Beam",
            Self::Block => "Block",
            Self::Underline => "Underline",
            Self::BlinkingBeam => "Blinking Beam",
            Self::BlinkingBlock => "Blinking Block",
            Self::BlinkingUnderline => "Blinking Underline",
        }
    }

    pub fn all() -> &'static [Self] {
        Self::ALL
    }

    pub fn name() -> &'static str {
        Self::NAME
    }
}

// ========== PICKER VARIANTS ==========

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum PickerVariant {
    #[default]
    Quake,
    Telescope,
    Ivy,
    Minimal,
}

impl std::str::FromStr for PickerVariant {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "quake" => Ok(Self::Quake),
            "telescope" => Ok(Self::Telescope),
            "ivy" => Ok(Self::Ivy),
            "minimal" => Ok(Self::Minimal),
            _ => Err(Error),
        }
    }
}

impl PickerVariant {
    pub const ALL: &'static [Self] = &[Self::Quake, Self::Telescope, Self::Ivy, Self::Minimal];
    pub const NAME: &'static str = "picker";

    pub fn value(&self) -> &'static str {
        match self {
            Self::Quake => "quake",
            Self::Telescope => "telescope",
            Self::Ivy => "ivy",
            Self::Minimal => "minimal",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Quake => "Quake",
            Self::Telescope => "Telescope",
            Self::Ivy => "Ivy",
            Self::Minimal => "Minimal",
        }
    }

    pub fn all() -> &'static [Self] {
        Self::ALL
    }

    pub fn name() -> &'static str {
        Self::NAME
    }
}

// ========== RESULTS VARIANTS ==========

#[derive(Debug, Clone, Default, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum ResultsVariant {
    #[default]
    Graph,
    Neofetch,
    Minimal,
}

impl std::str::FromStr for ResultsVariant {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "graph" => Ok(Self::Graph),
            "neofetch" => Ok(Self::Neofetch),
            "minimal" => Ok(Self::Minimal),
            _ => Err(Error),
        }
    }
}

impl ResultsVariant {
    pub const ALL: &'static [Self] = &[Self::Graph, Self::Neofetch, Self::Minimal];
    pub const NAME: &'static str = "results";

    pub fn value(&self) -> &'static str {
        match self {
            Self::Graph => "graph",
            Self::Neofetch => "neofetch",
            Self::Minimal => "minimal",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Graph => "Graph",
            Self::Neofetch => "Neofetch",
            Self::Minimal => "Minimal",
        }
    }

    pub fn all() -> &'static [Self] {
        Self::ALL
    }

    pub fn name() -> &'static str {
        Self::NAME
    }
}

pub trait Variant:
    Copy + Default + PartialEq + Eq + std::fmt::Debug + FromStr<Err = Error>
{
    fn all() -> &'static [&'static str];
    fn value(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn name() -> &'static str;
}

// Implement the trait for our variants
impl Variant for CursorVariant {
    fn all() -> &'static [&'static str] {
        &[
            "beam",
            "block",
            "underline",
            "blinking-beam",
            "blinking-block",
            "blinking-underline",
        ]
    }

    fn value(&self) -> &'static str {
        self.value()
    }

    fn label(&self) -> &'static str {
        self.label()
    }

    fn name() -> &'static str {
        Self::NAME
    }
}

impl Variant for PickerVariant {
    fn all() -> &'static [&'static str] {
        &["quake", "telescope", "ivy", "minimal"]
    }

    fn value(&self) -> &'static str {
        self.value()
    }

    fn label(&self) -> &'static str {
        self.label()
    }

    fn name() -> &'static str {
        Self::NAME
    }
}

impl Variant for ResultsVariant {
    fn all() -> &'static [&'static str] {
        &["graph", "neofetch", "minimal"]
    }

    fn value(&self) -> &'static str {
        self.value()
    }

    fn label(&self) -> &'static str {
        self.label()
    }

    fn name() -> &'static str {
        Self::NAME
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_variant() {
        let variant = CursorVariant::Beam;
        assert_eq!(variant.value(), "beam");
        assert_eq!(variant.label(), "Beam");
        assert_eq!(CursorVariant::name(), "cursor");
        assert_eq!(CursorVariant::default(), CursorVariant::Block);
    }

    #[test]
    fn test_picker_variant() {
        let variant = PickerVariant::Quake;
        assert_eq!(variant.value(), "quake");
        assert_eq!(variant.label(), "Quake");
        assert_eq!(PickerVariant::name(), "picker");
        assert_eq!(PickerVariant::default(), PickerVariant::Quake);
    }

    #[test]
    fn test_results_variant() {
        let variant = ResultsVariant::Graph;
        assert_eq!(variant.value(), "graph");
        assert_eq!(variant.label(), "Graph");
        assert_eq!(ResultsVariant::name(), "results");
        assert_eq!(ResultsVariant::default(), ResultsVariant::Graph);
    }

    #[test]
    fn test_parsing() {
        assert_eq!("beam".parse::<CursorVariant>(), Ok(CursorVariant::Beam));
        assert_eq!("QUAKE".parse::<PickerVariant>(), Ok(PickerVariant::Quake));
        assert_eq!(
            "minimal".parse::<ResultsVariant>(),
            Ok(ResultsVariant::Minimal)
        );
        assert!("invalid".parse::<CursorVariant>().is_err());
    }

    #[test]
    fn test_all_variants() {
        assert_eq!(CursorVariant::all().len(), 6);
        assert_eq!(PickerVariant::all().len(), 4);
        assert_eq!(ResultsVariant::all().len(), 3);
    }
}
