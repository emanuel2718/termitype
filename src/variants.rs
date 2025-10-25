use crossterm::cursor::SetCursorStyle;
use serde::{Deserialize, Serialize};
use std::fmt::Error;

// ========== CURSOR VARIANTS ==========

/// Represents different cursor styles available.
///
/// This enum provides various cursor appearances that can be set.
///
/// # Examples
///
/// ```
/// use termitype::variants::CursorVariant;
///
/// let cursor = CursorVariant::Beam;
/// assert_eq!(cursor.value(), "beam");
/// ```
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum CursorVariant {
    /// A solid block cursor (default).
    Block,
    /// A vertical beam cursor.
    #[default]
    Beam,
    /// An underline cursor.
    Underline,
    /// A blinking vertical beam cursor.
    BlinkingBeam,
    /// A blinking solid block cursor.
    BlinkingBlock,
    /// A blinking underline cursor.
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

    pub fn all() -> &'static [Self] {
        Self::ALL
    }

    pub fn name() -> &'static str {
        Self::NAME
    }

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

    /// Converts the `CursorVariant` to the corresponding `crossterm::cursor::SetCursorStyle`.
    ///
    /// # Examples
    ///
    /// ```
    /// use termitype::variants::CursorVariant;
    /// use crossterm::cursor::SetCursorStyle;
    ///
    /// let cursor = CursorVariant::Block;
    /// let style = cursor.to_crossterm();
    /// assert_eq!(style, SetCursorStyle::SteadyBlock);
    /// ```
    pub fn to_crossterm(&self) -> SetCursorStyle {
        match self {
            Self::Beam => SetCursorStyle::SteadyBar,
            Self::Block => SetCursorStyle::SteadyBlock,
            Self::Underline => SetCursorStyle::SteadyUnderScore,
            Self::BlinkingBeam => SetCursorStyle::BlinkingBar,
            Self::BlinkingBlock => SetCursorStyle::BlinkingBlock,
            Self::BlinkingUnderline => SetCursorStyle::BlinkingUnderScore,
        }
    }
}

// ========== PICKER VARIANTS ==========

/// Represents different picker styles for selecting items in the application.
///
///
/// # Examples
///
/// ```
/// use termitype::variants::PickerVariant;
///
/// let picker = PickerVariant::Quake;
/// assert_eq!(picker.value(), "quake");
/// ```
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum PickerVariant {
    /// Floating menu just like Telescopic johnson does
    #[default]
    Telescope,
    /// Opens from the top a la quake terminal style, hence the name
    Quake,
    /// Opens from the bottom.
    Ivy,
    /// Telescope style picker without footer and preview folds/splits
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

    pub fn all() -> &'static [Self] {
        Self::ALL
    }

    pub fn name() -> &'static str {
        Self::NAME
    }

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
}

// ========== RESULTS VARIANTS ==========

/// Represents different styles for displaying results in the application.
///
/// Results variants define how results or output are presented to the user,
/// such as graphical representations or minimal text displays.
///
/// # Examples
///
/// ```
/// use termitype::variants::ResultsVariant;
///
/// let results = ResultsVariant::Graph;
/// assert_eq!(results.value(), "graph");
/// ```
#[derive(Debug, Clone, Default, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum ResultsVariant {
    /// Minimal text display.
    Minimal,
    /// Graphical display style (default).
    #[default]
    Graph,
    /// Neofetch-style display.
    Neofetch,
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

    pub fn all() -> &'static [Self] {
        Self::ALL
    }

    pub fn name() -> &'static str {
        Self::NAME
    }

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
        assert_eq!(CursorVariant::default(), CursorVariant::Beam);
    }

    #[test]
    fn test_picker_variant() {
        let variant = PickerVariant::Quake;
        assert_eq!(variant.value(), "quake");
        assert_eq!(variant.label(), "Quake");
        assert_eq!(PickerVariant::name(), "picker");
        assert_eq!(PickerVariant::default(), PickerVariant::Telescope);
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
