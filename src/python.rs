//! Python Bindings for Urban Classifier
//!
//! This module provides PyO3 bindings to expose the urban classifier
//! functionality to Python. It wraps the Rust implementation in a
//! Python-friendly interface using Polars DataFrames.
//!
//! # Main Components
//!
//! - `PyUrbanClassifier`: Python wrapper around the Rust classifier
//! - Error conversion from Rust to Python exceptions
//! - Module initialization with metadata
//!
//! # Python API
//!
//! The module exposes:
//! - `PyUrbanClassifier` class with classification methods
//! - Static method to get LCZ information
//! - DataFrame validation utilities

#![allow(non_local_definitions)]

use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;
use std::collections::HashMap;

use crate::classifier::UrbanClassifier;
use crate::error::ClassifierError;

/// Python wrapper for the UrbanClassifier
#[pyclass]
pub struct PyUrbanClassifier {
    inner: UrbanClassifier,
}

#[pymethods]
impl PyUrbanClassifier {
    #[new]
    fn new(wudapt_path: &str) -> PyResult<Self> {
        let inner = UrbanClassifier::new(wudapt_path).map_err(convert_classifier_error_to_py)?;
        Ok(PyUrbanClassifier { inner })
    }

    /// Classify geographic coordinates using WUDAPT LCZ data.
    ///
    /// Parameters:
    /// - df: Polars DataFrame containing station data
    /// - station_id_col: Name of the column containing station IDs
    /// - lon_col: Name of the column containing longitude values
    /// - lat_col: Name of the column containing latitude values  
    /// - overrides: Optional dict mapping station IDs to LCZ codes for manual overrides
    ///
    /// Returns:
    /// Polars DataFrame with additional columns:
    /// - lcz_code: Numeric LCZ code (1-17)
    /// - lcz_name: Full descriptive name of the LCZ class
    /// - simple_class: Simplified category (Urban/Suburban/Rural)
    fn run_classification(
        &self,
        df: PyDataFrame,
        station_id_col: &str,
        lon_col: &str,
        lat_col: &str,
        overrides: Option<HashMap<String, u8>>,
    ) -> PyResult<PyDataFrame> {
        let result_df = self
            .inner
            .run_classification(&df.0, station_id_col, lon_col, lat_col, overrides.as_ref())
            .map_err(convert_classifier_error_to_py)?;

        Ok(PyDataFrame(result_df))
    }

    /// Get information about the LCZ classification system.
    ///
    /// Returns a dictionary containing:
    /// - codes: List of valid LCZ codes (1-17)
    /// - names: List of corresponding descriptive names
    /// - categories: List of simplified categories
    #[staticmethod]
    fn get_lcz_info() -> PyResult<HashMap<String, Vec<String>>> {
        use crate::lcz::Lcz;

        let mut codes = Vec::new();
        let mut names = Vec::new();
        let mut categories = Vec::new();

        for code in 1..=17 {
            let lcz = Lcz::from_code(code);
            codes.push(code.to_string());
            names.push(lcz.full_name().to_string());
            categories.push(lcz.simple_category().as_ref().to_string());
        }

        let mut result = HashMap::new();
        result.insert("codes".to_string(), codes);
        result.insert("names".to_string(), names);
        result.insert("categories".to_string(), categories);

        Ok(result)
    }

    /// Validate a DataFrame schema for compatibility with classification.
    ///
    /// Parameters:
    /// - df: Polars DataFrame to validate
    /// - station_id_col: Name of the station ID column
    /// - lon_col: Name of the longitude column
    /// - lat_col: Name of the latitude column
    ///
    /// Returns:
    /// True if the DataFrame is valid, raises exception if not.
    fn validate_dataframe(
        &self,
        df: PyDataFrame,
        station_id_col: &str,
        lon_col: &str,
        lat_col: &str,
    ) -> PyResult<bool> {
        // Use a dummy classification to validate the schema
        // This is a bit inefficient but ensures we use the same validation logic
        let _result = self
            .inner
            .run_classification(&df.0, station_id_col, lon_col, lat_col, None)
            .map_err(convert_classifier_error_to_py)?;

        Ok(true)
    }
}

/// Convert Rust ClassifierError to appropriate Python exceptions
fn convert_classifier_error_to_py(error: ClassifierError) -> PyErr {
    match error {
        ClassifierError::FileNotFound { path } => {
            pyo3::exceptions::PyFileNotFoundError::new_err(format!("File not found: {}", path))
        }
        ClassifierError::ColumnNotFound { column } => {
            pyo3::exceptions::PyKeyError::new_err(format!("Column not found: {}", column))
        }
        ClassifierError::InvalidCoordinate { lon, lat } => pyo3::exceptions::PyValueError::new_err(
            format!("Invalid coordinate: longitude={}, latitude={}", lon, lat),
        ),
        ClassifierError::SchemaValidation { message } => pyo3::exceptions::PyValueError::new_err(
            format!("Schema validation failed: {}", message),
        ),
        ClassifierError::GdalError { message } => {
            pyo3::exceptions::PyRuntimeError::new_err(format!("GDAL error: {}", message))
        }
        ClassifierError::CoordinateTransform { message } => {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Coordinate transformation failed: {}",
                message
            ))
        }
        ClassifierError::RasterSampling {
            pixel,
            line,
            message,
        } => pyo3::exceptions::PyRuntimeError::new_err(format!(
            "Raster sampling failed at ({}, {}): {}",
            pixel, line, message
        )),
        ClassifierError::OverrideApplication {
            station_id,
            message,
        } => pyo3::exceptions::PyValueError::new_err(format!(
            "Override application failed for station {}: {}",
            station_id, message
        )),
        ClassifierError::Polars(e) => {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Polars error: {}", e))
        }
        ClassifierError::Io(e) => pyo3::exceptions::PyIOError::new_err(format!("I/O error: {}", e)),
    }
}

/// Urban classifier module for Local Climate Zone (LCZ) classification.
///
/// This module provides functionality to classify geographic coordinates
/// according to the Local Climate Zone system using WUDAPT global data.
#[pymodule]
#[pyo3(name = "urban_classifier")]
pub fn urban_classifier_module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyUrbanClassifier>()?;

    // Add module-level constants
    m.add("__version__", "0.1.0")?;
    m.add("__author__", "Richard Lyon")?;
    m.add(
        "__description__",
        "High-performance urban classification using Local Climate Zone (LCZ) system",
    )?;

    Ok(())
}
