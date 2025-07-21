//! Local Climate Zone (LCZ) Classification System
//!
//! This module implements the Local Climate Zone classification system developed by
//! Stewart and Oke (2012) for urban climate studies. The LCZ system categorizes
//! landscapes into 17 standard classes based on surface cover, structure, fabric,
//! and human activity.
//!
//! # LCZ Classes
//!
//! ## Built Types (1-10)
//! - **Compact (1-3)**: Densely built areas with little vegetation
//! - **Open (4-6)**: Less densely built with more vegetation
//! - **Sparse/Industrial (7-10)**: Low density or specialized use
//!
//! ## Land Cover Types (11-17)
//! - **Vegetated (11-14)**: Natural surfaces with vegetation
//! - **Bare (15-17)**: Natural surfaces without vegetation
//!
//! # Simple Categories
//!
//! For simplified analysis, LCZ classes are grouped into:
//! - **Urban**: Classes 1-6 (compact and open built areas)
//! - **Suburban**: Classes 7-10 (sparse built and industrial)
//! - **Rural**: Classes 11-17 (natural land cover)

use serde::{Deserialize, Serialize};

/// Local Climate Zone classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lcz {
    // Urban types (1-10)
    CompactHighRise,    // 1
    CompactMidRise,     // 2
    CompactLowRise,     // 3
    OpenHighRise,       // 4
    OpenMidRise,        // 5
    OpenLowRise,        // 6
    LightweightLowRise, // 7
    LargeLowRise,       // 8
    SparselyBuilt,      // 9
    HeavyIndustry,      // 10

    // Natural types (11-17)
    DenseTrees,     // 11 (A)
    ScatteredTrees, // 12 (B)
    BushScrub,      // 13 (C)
    LowPlants,      // 14 (D)
    BareRockPaved,  // 15 (E)
    BareSoilSand,   // 16 (F)
    Water,          // 17 (G)

    // Unknown/invalid codes
    Unknown(u8),
}

/// Simplified urban/rural classification categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LczCategory {
    /// Dense urban areas (LCZ 1-6)
    Urban,
    /// Sparse/industrial areas (LCZ 7-10)
    Suburban,
    /// Natural/vegetated areas (LCZ 11-17)
    Rural,
}

impl Lcz {
    /// Convert a numeric LCZ code (1-17) to the corresponding enum variant
    pub fn from_code(code: u8) -> Self {
        match code {
            1 => Lcz::CompactHighRise,
            2 => Lcz::CompactMidRise,
            3 => Lcz::CompactLowRise,
            4 => Lcz::OpenHighRise,
            5 => Lcz::OpenMidRise,
            6 => Lcz::OpenLowRise,
            7 => Lcz::LightweightLowRise,
            8 => Lcz::LargeLowRise,
            9 => Lcz::SparselyBuilt,
            10 => Lcz::HeavyIndustry,
            11 => Lcz::DenseTrees,
            12 => Lcz::ScatteredTrees,
            13 => Lcz::BushScrub,
            14 => Lcz::LowPlants,
            15 => Lcz::BareRockPaved,
            16 => Lcz::BareSoilSand,
            17 => Lcz::Water,
            _ => Lcz::Unknown(code),
        }
    }

    /// Convert the enum variant back to its numeric LCZ code
    pub fn to_code(&self) -> u8 {
        match self {
            Lcz::CompactHighRise => 1,
            Lcz::CompactMidRise => 2,
            Lcz::CompactLowRise => 3,
            Lcz::OpenHighRise => 4,
            Lcz::OpenMidRise => 5,
            Lcz::OpenLowRise => 6,
            Lcz::LightweightLowRise => 7,
            Lcz::LargeLowRise => 8,
            Lcz::SparselyBuilt => 9,
            Lcz::HeavyIndustry => 10,
            Lcz::DenseTrees => 11,
            Lcz::ScatteredTrees => 12,
            Lcz::BushScrub => 13,
            Lcz::LowPlants => 14,
            Lcz::BareRockPaved => 15,
            Lcz::BareSoilSand => 16,
            Lcz::Water => 17,
            Lcz::Unknown(code) => *code,
        }
    }

    /// Get the human-readable full name of the LCZ class
    pub fn full_name(&self) -> &'static str {
        match self {
            Lcz::CompactHighRise => "Compact high-rise",
            Lcz::CompactMidRise => "Compact midrise",
            Lcz::CompactLowRise => "Compact low-rise",
            Lcz::OpenHighRise => "Open high-rise",
            Lcz::OpenMidRise => "Open midrise",
            Lcz::OpenLowRise => "Open low-rise",
            Lcz::LightweightLowRise => "Lightweight low-rise",
            Lcz::LargeLowRise => "Large low-rise",
            Lcz::SparselyBuilt => "Sparsely built",
            Lcz::HeavyIndustry => "Heavy industry",
            Lcz::DenseTrees => "Dense trees",
            Lcz::ScatteredTrees => "Scattered trees",
            Lcz::BushScrub => "Bush, scrub",
            Lcz::LowPlants => "Low plants",
            Lcz::BareRockPaved => "Bare rock or paved",
            Lcz::BareSoilSand => "Bare soil or sand",
            Lcz::Water => "Water",
            Lcz::Unknown(_) => "Unknown",
        }
    }

    /// Get the simplified urban/suburban/rural category for this LCZ class
    pub fn simple_category(&self) -> LczCategory {
        match self {
            // Urban types (1-6)
            Lcz::CompactHighRise
            | Lcz::CompactMidRise
            | Lcz::CompactLowRise
            | Lcz::OpenHighRise
            | Lcz::OpenMidRise
            | Lcz::OpenLowRise => LczCategory::Urban,

            // Suburban types (7-10)
            Lcz::LightweightLowRise
            | Lcz::LargeLowRise
            | Lcz::SparselyBuilt
            | Lcz::HeavyIndustry => LczCategory::Suburban,

            // Rural/Natural types (11-17)
            Lcz::DenseTrees
            | Lcz::ScatteredTrees
            | Lcz::BushScrub
            | Lcz::LowPlants
            | Lcz::BareRockPaved
            | Lcz::BareSoilSand
            | Lcz::Water => LczCategory::Rural,

            // Unknown
            Lcz::Unknown(_) => LczCategory::Rural, // Default to rural for unknown
        }
    }
}

impl AsRef<str> for LczCategory {
    fn as_ref(&self) -> &str {
        match self {
            LczCategory::Urban => "Urban",
            LczCategory::Suburban => "Suburban",
            LczCategory::Rural => "Rural",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that LCZ codes can be converted to enum variants and back
    #[test]
    fn test_lcz_code_conversion() {
        // Test all valid codes
        for code in 1..=17 {
            let lcz = Lcz::from_code(code);
            assert_eq!(
                lcz.to_code(),
                code,
                "Code conversion failed for LCZ {}",
                code
            );
        }
    }

    /// Test handling of invalid/unknown LCZ codes
    #[test]
    fn test_unknown_lcz() {
        let unknown = Lcz::from_code(99);
        assert_eq!(unknown, Lcz::Unknown(99));
        assert_eq!(unknown.to_code(), 99);
        assert_eq!(unknown.full_name(), "Unknown");
    }

    /// Test correct assignment of LCZ classes to simplified categories
    #[test]
    fn test_lcz_categories() {
        // Test urban types
        assert_eq!(Lcz::CompactHighRise.simple_category(), LczCategory::Urban);
        assert_eq!(Lcz::OpenLowRise.simple_category(), LczCategory::Urban);

        // Test suburban types
        assert_eq!(
            Lcz::LightweightLowRise.simple_category(),
            LczCategory::Suburban
        );
        assert_eq!(Lcz::HeavyIndustry.simple_category(), LczCategory::Suburban);

        // Test rural types
        assert_eq!(Lcz::DenseTrees.simple_category(), LczCategory::Rural);
        assert_eq!(Lcz::Water.simple_category(), LczCategory::Rural);
    }

    /// Test string representation of simplified categories
    #[test]
    fn test_category_as_ref() {
        assert_eq!(LczCategory::Urban.as_ref(), "Urban");
        assert_eq!(LczCategory::Suburban.as_ref(), "Suburban");
        assert_eq!(LczCategory::Rural.as_ref(), "Rural");
    }
}
