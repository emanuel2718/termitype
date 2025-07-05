use std::fmt::{Display, Formatter};
use std::str::FromStr;

// ========== TERMI STYLE TRAIT ==========
pub trait TermiStyle:
    Copy + Default + PartialEq + Eq + std::fmt::Debug + FromStr<Err = TermiStyleParseError>
{
    /// Convert enum variant to string representation
    fn as_str(&self) -> &'static str;

    /// Get all available options for this style
    fn all() -> &'static [&'static str];

    /// Get display label for a string value
    fn label_from_str(s: &str) -> &'static str;

    /// Get the style name
    fn name() -> &'static str;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TermiStyleParseError {
    pub invalid_input: String,
    pub style_name: &'static str,
    pub valid_options: &'static [&'static str],
}

impl Display for TermiStyleParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid {} TermiStyle: '{}'. Valid options are: {}",
            self.style_name,
            self.invalid_input,
            self.valid_options.join(", ")
        )
    }
}

impl std::error::Error for TermiStyleParseError {}

impl TermiStyleParseError {
    pub fn new(
        invalid_input: String,
        style_name: &'static str,
        valid_options: &'static [&'static str],
    ) -> Self {
        Self {
            invalid_input,
            style_name,
            valid_options,
        }
    }
}

// ========== PICKER STYLE ==========

/// Picker style for the style of menu
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PickerStyle {
    /// Opens from the top a la quake terminal style, hence the name
    Quake,
    /// Floating menu just like Telescopic johnson does
    Telescope,
    /// Opens from the bottom
    Ivy,
    /// Telescope style picker without footer and preview folds/splits
    Minimal,
}

impl PickerStyle {
    pub const ALL: &'static [&'static str] = &["quake", "telescope", "ivy", "minimal"];
    pub const NAME: &'static str = "picker";
}

impl Default for PickerStyle {
    fn default() -> Self {
        Self::Quake
    }
}

impl FromStr for PickerStyle {
    type Err = TermiStyleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "quake" => Ok(Self::Quake),
            "telescope" => Ok(Self::Telescope),
            "ivy" => Ok(Self::Ivy),
            "minimal" => Ok(Self::Minimal),
            _ => Err(TermiStyleParseError::new(
                s.to_string(),
                Self::NAME,
                Self::ALL,
            )),
        }
    }
}

impl TermiStyle for PickerStyle {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Quake => "quake",
            Self::Telescope => "telescope",
            Self::Ivy => "ivy",
            Self::Minimal => "minimal",
        }
    }

    fn all() -> &'static [&'static str] {
        Self::ALL
    }

    fn label_from_str(s: &str) -> &'static str {
        match s {
            "quake" => "Quake",
            "telescope" => "Telescope",
            "ivy" => "Ivy",
            "minimal" => "Minimal",
            _ => "Unknown picker style",
        }
    }

    fn name() -> &'static str {
        Self::NAME
    }
}

// ========== RESULTS STYLE ==========

/// Style of the results
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ResultsStyle {
    /// Graph-based results display
    Graph,
    /// Neofetch-inspired results display
    Neofetch,
    /// Minimal results display
    Minimal,
}

impl ResultsStyle {
    pub const ALL: &'static [&'static str] = &["graph", "neofetch", "minimal"];
    pub const NAME: &'static str = "results";
}

impl Default for ResultsStyle {
    fn default() -> Self {
        Self::Graph
    }
}

impl FromStr for ResultsStyle {
    type Err = TermiStyleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "graph" => Ok(Self::Graph),
            "neofetch" => Ok(Self::Neofetch),
            "minimal" => Ok(Self::Minimal),
            _ => Err(TermiStyleParseError::new(
                s.to_string(),
                Self::NAME,
                Self::ALL,
            )),
        }
    }
}

impl TermiStyle for ResultsStyle {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Graph => "graph",
            Self::Neofetch => "neofetch",
            Self::Minimal => "minimal",
        }
    }

    fn all() -> &'static [&'static str] {
        Self::ALL
    }

    fn label_from_str(s: &str) -> &'static str {
        match s {
            "graph" => "Graph",
            "neofetch" => "Neofetch",
            "minimal" => "Minimal",
            _ => "Unknown results style",
        }
    }

    fn name() -> &'static str {
        Self::NAME
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_picker_style_from_str() {
        assert_eq!("quake".parse::<PickerStyle>(), Ok(PickerStyle::Quake));
        assert_eq!(
            "telescope".parse::<PickerStyle>(),
            Ok(PickerStyle::Telescope)
        );
        assert_eq!("ivy".parse::<PickerStyle>(), Ok(PickerStyle::Ivy));
        assert_eq!("minimal".parse::<PickerStyle>(), Ok(PickerStyle::Minimal));
        assert_eq!("QUAKE".parse::<PickerStyle>(), Ok(PickerStyle::Quake));
        assert!("invalid".parse::<PickerStyle>().is_err());

        let err = "invalid".parse::<PickerStyle>().unwrap_err();
        assert!(err.to_string().contains("Invalid picker TermiStyle"));
        assert!(err.to_string().contains("quake"));
    }

    #[test]
    fn test_results_style_from_str() {
        assert_eq!("graph".parse::<ResultsStyle>(), Ok(ResultsStyle::Graph));
        assert_eq!("minimal".parse::<ResultsStyle>(), Ok(ResultsStyle::Minimal));
        assert_eq!(
            "neofetch".parse::<ResultsStyle>(),
            Ok(ResultsStyle::Neofetch)
        );
        assert_eq!("GRAPH".parse::<ResultsStyle>(), Ok(ResultsStyle::Graph));
        assert!("invalid".parse::<ResultsStyle>().is_err());

        let err = "invalid".parse::<ResultsStyle>().unwrap_err();
        assert!(err.to_string().contains("Invalid results TermiStyle"));
        assert!(err.to_string().contains("graph"));
    }

    #[test]
    fn test_picker_style_as_str() {
        assert_eq!(PickerStyle::Quake.as_str(), "quake");
        assert_eq!(PickerStyle::Telescope.as_str(), "telescope");
        assert_eq!(PickerStyle::Ivy.as_str(), "ivy");
        assert_eq!(PickerStyle::Minimal.as_str(), "minimal");
    }

    #[test]
    fn test_results_style_as_str() {
        assert_eq!(ResultsStyle::Graph.as_str(), "graph");
        assert_eq!(ResultsStyle::Minimal.as_str(), "minimal");
        assert_eq!(ResultsStyle::Neofetch.as_str(), "neofetch");
    }

    #[test]
    fn test_picker_style_all() {
        let all = PickerStyle::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&"quake"));
        assert!(all.contains(&"telescope"));
        assert!(all.contains(&"ivy"));
        assert!(all.contains(&"minimal"));
    }

    #[test]
    fn test_results_style_all() {
        let all = ResultsStyle::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&"graph"));
        assert!(all.contains(&"minimal"));
        assert!(all.contains(&"neofetch"));
    }

    #[test]
    fn test_picker_style_label_from_str() {
        assert_eq!(PickerStyle::label_from_str("quake"), "Quake");
        assert_eq!(PickerStyle::label_from_str("telescope"), "Telescope");
        assert_eq!(PickerStyle::label_from_str("ivy"), "Ivy");
        assert_eq!(PickerStyle::label_from_str("minimal"), "Minimal");
        assert_eq!(
            PickerStyle::label_from_str("invalid"),
            "Unknown picker style"
        );
    }

    #[test]
    fn test_results_style_label_from_str() {
        assert_eq!(ResultsStyle::label_from_str("graph"), "Graph");
        assert_eq!(ResultsStyle::label_from_str("minimal"), "Minimal");
        assert_eq!(ResultsStyle::label_from_str("neofetch"), "Neofetch");
        assert_eq!(
            ResultsStyle::label_from_str("invalid"),
            "Unknown results style"
        );
    }

    #[test]
    fn test_results_style_parsing() {
        assert_eq!(
            "graph".parse::<ResultsStyle>().unwrap(),
            ResultsStyle::Graph
        );
        assert_eq!(
            "minimal".parse::<ResultsStyle>().unwrap(),
            ResultsStyle::Minimal
        );
        assert_eq!(
            "neofetch".parse::<ResultsStyle>().unwrap(),
            ResultsStyle::Neofetch
        );
        assert!("invalid_picker_yaya".parse::<ResultsStyle>().is_err());
    }

    #[test]
    fn test_defaults() {
        assert_eq!(PickerStyle::default(), PickerStyle::Quake);
        assert_eq!(ResultsStyle::default(), ResultsStyle::Graph);
    }

    #[test]
    fn test_case_insensitive_parsing() {
        assert_eq!(
            "TELESCOPE".parse::<PickerStyle>(),
            Ok(PickerStyle::Telescope)
        );
        assert_eq!("Ivy".parse::<PickerStyle>(), Ok(PickerStyle::Ivy));
        assert_eq!(
            "NEOFETCH".parse::<ResultsStyle>(),
            Ok(ResultsStyle::Neofetch)
        );
    }

    #[test]
    fn test_style_names() {
        assert_eq!(PickerStyle::name(), "picker");
        assert_eq!(ResultsStyle::name(), "results");
    }
}
