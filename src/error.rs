//! Error Types for Urban Classifier
//!
//! This module defines custom error types for the urban classifier library,
//! providing detailed error information for various failure scenarios.
//!
//! # Error Categories
//!
//! - **File Operations**: File not found errors
//! - **GDAL Operations**: GeoTIFF reading and raster operations
//! - **Data Validation**: DataFrame schema and coordinate validation
//! - **Spatial Operations**: Coordinate transformation and raster sampling
//! - **External Libraries**: Wrapper errors for Polars and I/O operations

use thiserror::Error;

/// Type alias for Results with ClassifierError
pub type Result<T> = std::result::Result<T, ClassifierError>;

/// Main error type for the urban classifier library
#[derive(Error, Debug)]
pub enum ClassifierError {
    /// The specified file path does not exist
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    /// GDAL library error during GeoTIFF operations
    #[error("GDAL error: {message}")]
    GdalError { message: String },

    /// Required column missing from input DataFrame
    #[error("Column not found in DataFrame: {column}")]
    ColumnNotFound { column: String },

    /// Invalid geographic coordinates (outside valid range)
    #[error("Invalid coordinate: lon={lon}, lat={lat}")]
    InvalidCoordinate { lon: f64, lat: f64 },

    /// Failed to transform coordinates between spatial reference systems
    #[error("Coordinate transformation failed: {message}")]
    CoordinateTransform { message: String },

    /// Failed to sample raster value at specified pixel location
    #[error("Raster sampling failed at pixel ({pixel}, {line}): {message}")]
    RasterSampling {
        pixel: isize,
        line: isize,
        message: String,
    },

    /// DataFrame does not meet required schema specifications
    #[error("DataFrame schema validation failed: {message}")]
    SchemaValidation { message: String },

    /// Failed to apply manual LCZ override for a station
    #[error("Override application failed for station {station_id}: {message}")]
    OverrideApplication { station_id: String, message: String },

    /// Wrapper for Polars DataFrame errors
    #[error("Polars error: {0}")]
    Polars(#[from] polars::error::PolarsError),

    /// Wrapper for standard I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<gdal::errors::GdalError> for ClassifierError {
    fn from(err: gdal::errors::GdalError) -> Self {
        ClassifierError::GdalError {
            message: err.to_string(),
        }
    }
}
