//! Urban Classifier Library
//!
//! This library provides functionality to classify geographic locations (typically weather stations)
//! based on their urban/rural characteristics using the Local Climate Zone (LCZ) classification system
//! and WUDAPT (World Urban Database and Access Portal Tools) data.
//!
//! # Main Components
//!
//! - **Classifier**: The main `UrbanClassifier` struct that performs classification
//! - **LCZ System**: Enums and types representing the 17 Local Climate Zone categories
//! - **Spatial Operations**: GeoTIFF reading and coordinate transformation utilities
//! - **Error Handling**: Custom error types for robust error reporting
//! - **Python Bindings**: Optional PyO3 bindings for Python integration
//!
//! # Usage
//!
//! ```no_run
//! use urban_classifier::UrbanClassifier;
//! use polars::prelude::*;
//!
//! // Load WUDAPT GeoTIFF file
//! let classifier = UrbanClassifier::new("path/to/wudapt.tif").unwrap();
//!
//! // Create DataFrame with station data
//! let df = df! {
//!     "station_id" => ["A", "B"],
//!     "longitude" => [-0.1278, 2.3522],
//!     "latitude" => [51.5074, 48.8566],
//! }.unwrap();
//!
//! // Run classification
//! let result = classifier.run_classification(
//!     &df,
//!     "station_id",
//!     "longitude",
//!     "latitude",
//!     None
//! ).unwrap();
//! ```

pub mod classifier;
pub mod error;
pub mod lcz;
pub mod spatial;

#[cfg(feature = "python")]
pub mod python;

pub use classifier::UrbanClassifier;
pub use error::ClassifierError;
pub use lcz::{Lcz, LczCategory};

// Re-export for Python bindings
#[cfg(feature = "python")]
pub use python::urban_classifier_module;
