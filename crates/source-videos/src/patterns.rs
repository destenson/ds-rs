use crate::error::{Result, SourceVideoError};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TestPattern {
    Smpte,
    Snow,
    Black,
    White,
    Red,
    Green,
    Blue,
    Checkers1,
    Checkers2,
    Checkers4,
    Checkers8,
    Circular,
    Blink,
    Smpte75,
    ZonePlate,
    Gamut,
    ChromaZonePlate,
    SolidColor,
    Ball,
    Smpte100,
    Bar,
    PinWheel,
    Spokes,
    Gradient,
    Colors,
}

impl TestPattern {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "smpte" => Ok(Self::Smpte),
            "snow" => Ok(Self::Snow),
            "black" => Ok(Self::Black),
            "white" => Ok(Self::White),
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "blue" => Ok(Self::Blue),
            "checkers-1" | "checkers1" => Ok(Self::Checkers1),
            "checkers-2" | "checkers2" => Ok(Self::Checkers2),
            "checkers-4" | "checkers4" => Ok(Self::Checkers4),
            "checkers-8" | "checkers8" => Ok(Self::Checkers8),
            "circular" => Ok(Self::Circular),
            "blink" => Ok(Self::Blink),
            "smpte75" | "smpte-75" => Ok(Self::Smpte75),
            "zone-plate" | "zoneplate" => Ok(Self::ZonePlate),
            "gamut" => Ok(Self::Gamut),
            "chroma-zone-plate" | "chromazoneplate" => Ok(Self::ChromaZonePlate),
            "solid-color" | "solidcolor" => Ok(Self::SolidColor),
            "ball" => Ok(Self::Ball),
            "smpte100" | "smpte-100" => Ok(Self::Smpte100),
            "bar" => Ok(Self::Bar),
            "pinwheel" | "pin-wheel" => Ok(Self::PinWheel),
            "spokes" => Ok(Self::Spokes),
            "gradient" => Ok(Self::Gradient),
            "colors" => Ok(Self::Colors),
            _ => Err(SourceVideoError::InvalidPattern(format!(
                "Unknown pattern: {}. Use 'list' to see available patterns.",
                s
            ))),
        }
    }

    pub fn to_gst_pattern(&self) -> i32 {
        match self {
            Self::Smpte => 0,
            Self::Snow => 1,
            Self::Black => 2,
            Self::White => 3,
            Self::Red => 4,
            Self::Green => 5,
            Self::Blue => 6,
            Self::Checkers1 => 7,
            Self::Checkers2 => 8,
            Self::Checkers4 => 9,
            Self::Checkers8 => 10,
            Self::Circular => 11,
            Self::Blink => 12,
            Self::Smpte75 => 13,
            Self::ZonePlate => 14,
            Self::Gamut => 15,
            Self::ChromaZonePlate => 16,
            Self::SolidColor => 17,
            Self::Ball => 18,
            Self::Smpte100 => 19,
            Self::Bar => 20,
            Self::PinWheel => 21,
            Self::Spokes => 22,
            Self::Gradient => 23,
            Self::Colors => 24,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Smpte => "SMPTE 100% color bars",
            Self::Snow => "Random noise pattern",
            Self::Black => "Solid black frame",
            Self::White => "Solid white frame",
            Self::Red => "Solid red frame",
            Self::Green => "Solid green frame",
            Self::Blue => "Solid blue frame",
            Self::Checkers1 => "1px checkers pattern",
            Self::Checkers2 => "2px checkers pattern",
            Self::Checkers4 => "4px checkers pattern",
            Self::Checkers8 => "8px checkers pattern",
            Self::Circular => "Circular pattern",
            Self::Blink => "Blinking black/white",
            Self::Smpte75 => "SMPTE 75% color bars",
            Self::ZonePlate => "Zone plate pattern for testing",
            Self::Gamut => "Color gamut pattern",
            Self::ChromaZonePlate => "Chroma zone plate",
            Self::SolidColor => "Solid color (configurable)",
            Self::Ball => "Moving ball animation",
            Self::Smpte100 => "SMPTE 100% color bars (alias)",
            Self::Bar => "Horizontal bar moving vertically",
            Self::PinWheel => "Pinwheel pattern",
            Self::Spokes => "Spokes pattern",
            Self::Gradient => "Gradient pattern",
            Self::Colors => "All colors pattern",
        }
    }

    pub fn use_case(&self) -> &str {
        match self {
            Self::Smpte | Self::Smpte75 | Self::Smpte100 => {
                "Standard broadcast test pattern for color calibration"
            }
            Self::Snow => "Testing noise handling and compression",
            Self::Black | Self::White => "Testing black/white level handling",
            Self::Red | Self::Green | Self::Blue => "Testing color channel processing",
            Self::Checkers1 | Self::Checkers2 | Self::Checkers4 | Self::Checkers8 => {
                "Testing resolution and aliasing"
            }
            Self::Ball | Self::Bar => "Testing motion detection and tracking",
            Self::Blink => "Testing temporal processing",
            Self::ZonePlate | Self::ChromaZonePlate => "Testing frequency response",
            Self::Gradient => "Testing color depth and banding",
            _ => "General testing purposes",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Smpte,
            Self::Snow,
            Self::Black,
            Self::White,
            Self::Red,
            Self::Green,
            Self::Blue,
            Self::Checkers1,
            Self::Checkers2,
            Self::Checkers4,
            Self::Checkers8,
            Self::Circular,
            Self::Blink,
            Self::Smpte75,
            Self::ZonePlate,
            Self::Gamut,
            Self::ChromaZonePlate,
            Self::SolidColor,
            Self::Ball,
            Self::Smpte100,
            Self::Bar,
            Self::PinWheel,
            Self::Spokes,
            Self::Gradient,
            Self::Colors,
        ]
    }

    pub fn animated_patterns() -> Vec<Self> {
        vec![Self::Ball, Self::Bar, Self::Blink, Self::Snow]
    }

    pub fn static_patterns() -> Vec<Self> {
        Self::all()
            .into_iter()
            .filter(|p| !Self::animated_patterns().contains(p))
            .collect()
    }
}

impl fmt::Display for TestPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct PatternRotator {
    patterns: Vec<TestPattern>,
    current_index: usize,
}

impl PatternRotator {
    pub fn new(patterns: Vec<TestPattern>) -> Self {
        Self {
            patterns,
            current_index: 0,
        }
    }

    pub fn all_patterns() -> Self {
        Self::new(TestPattern::all())
    }

    pub fn animated_only() -> Self {
        Self::new(TestPattern::animated_patterns())
    }

    pub fn static_only() -> Self {
        Self::new(TestPattern::static_patterns())
    }

    pub fn next(&mut self) -> TestPattern {
        if self.patterns.is_empty() {
            return TestPattern::Smpte;
        }

        let pattern = self.patterns[self.current_index];
        self.current_index = (self.current_index + 1) % self.patterns.len();
        pattern
    }

    pub fn current(&self) -> TestPattern {
        if self.patterns.is_empty() {
            TestPattern::Smpte
        } else {
            self.patterns[self.current_index]
        }
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_from_str() {
        assert_eq!(TestPattern::from_str("smpte").unwrap(), TestPattern::Smpte);
        assert_eq!(TestPattern::from_str("BALL").unwrap(), TestPattern::Ball);
        assert_eq!(
            TestPattern::from_str("zone-plate").unwrap(),
            TestPattern::ZonePlate
        );
        assert!(TestPattern::from_str("invalid").is_err());
    }

    #[test]
    fn test_pattern_to_gst() {
        assert_eq!(TestPattern::Smpte.to_gst_pattern(), 0);
        assert_eq!(TestPattern::Ball.to_gst_pattern(), 18);
        assert_eq!(TestPattern::Colors.to_gst_pattern(), 24);
    }

    #[test]
    fn test_pattern_rotator() {
        let mut rotator = PatternRotator::new(vec![
            TestPattern::Smpte,
            TestPattern::Ball,
            TestPattern::Snow,
        ]);

        assert_eq!(rotator.current(), TestPattern::Smpte);
        assert_eq!(rotator.next(), TestPattern::Smpte);
        assert_eq!(rotator.next(), TestPattern::Ball);
        assert_eq!(rotator.next(), TestPattern::Snow);
        assert_eq!(rotator.next(), TestPattern::Smpte);
    }

    #[test]
    fn test_animated_vs_static() {
        let animated = TestPattern::animated_patterns();
        assert!(animated.contains(&TestPattern::Ball));
        assert!(!animated.contains(&TestPattern::Smpte));

        let static_patterns = TestPattern::static_patterns();
        assert!(!static_patterns.contains(&TestPattern::Ball));
        assert!(static_patterns.contains(&TestPattern::Smpte));
    }
}
