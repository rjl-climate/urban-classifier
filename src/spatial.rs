//! Spatial Operations and Coordinate Transformations
//!
//! This module provides utilities for working with geospatial data, including:
//! - Coordinate system transformations (WGS84 to raster CRS)
//! - Converting geographic coordinates to pixel coordinates
//! - Sampling raster values at specific locations
//! - Validating geotransform parameters
//!
//! # Coordinate Systems
//!
//! The module handles transformations between:
//! - **WGS84 (EPSG:4326)**: Standard lat/lon coordinates
//! - **Raster CRS**: The coordinate system of the input GeoTIFF
//!
//! # Geotransform
//!
//! GDAL geotransform is a 6-element array:
//! - [0]: Top-left X coordinate
//! - [1]: Pixel width (X resolution)
//! - \[2\]: Rotation (0 for north-up images)
//! - [3]: Top-left Y coordinate
//! - \[4\]: Rotation (0 for north-up images)
//! - [5]: Pixel height (negative for north-up)

use crate::error::{ClassifierError, Result};
use gdal::raster::RasterBand;
use gdal::spatial_ref::{CoordTransform, SpatialRef};

/// Create a coordinate transformation from WGS84 to the raster's coordinate system
pub fn create_wgs84_to_raster_transform(raster_srs: &SpatialRef) -> Result<CoordTransform> {
    let wgs84 = SpatialRef::from_epsg(4326)?;

    CoordTransform::new(&wgs84, raster_srs).map_err(|e| ClassifierError::CoordinateTransform {
        message: format!("Failed to create coordinate transform: {}", e),
    })
}

/// Transform a single coordinate from WGS84 to the target coordinate system
///
/// # Arguments
/// * `lon` - Longitude in degrees (-180 to 180)
/// * `lat` - Latitude in degrees (-90 to 90)
/// * `transform` - The coordinate transformation to apply
///
/// # Returns
/// Transformed (x, y) coordinates in the target system
pub fn transform_coordinate(lon: f64, lat: f64, transform: &CoordTransform) -> Result<(f64, f64)> {
    // Validate input coordinates
    if !(-180.0..=180.0).contains(&lon) || !(-90.0..=90.0).contains(&lat) {
        return Err(ClassifierError::InvalidCoordinate { lon, lat });
    }

    let mut x = [lon];
    let mut y = [lat];
    let mut z = [0.0];

    transform
        .transform_coords(&mut x, &mut y, &mut z)
        .map_err(|e| ClassifierError::CoordinateTransform {
            message: format!("Failed to transform coordinates ({}, {}): {}", lon, lat, e),
        })?;

    Ok((x[0], y[0]))
}

/// Convert geographic coordinates to pixel coordinates using the geotransform
///
/// # Arguments
/// * `x` - X coordinate in the raster's coordinate system
/// * `y` - Y coordinate in the raster's coordinate system
/// * `geo_transform` - GDAL geotransform array
///
/// # Returns
/// (pixel, line) coordinates in the raster
pub fn geo_to_pixel(x: f64, y: f64, geo_transform: &[f64; 6]) -> (isize, isize) {
    // Apply inverse affine transformation
    // geo_transform: [x_origin, pixel_width, x_skew, y_origin, y_skew, pixel_height]
    // where pixel_height is typically negative

    let pixel = ((x - geo_transform[0]) / geo_transform[1]) as isize;
    let line = ((y - geo_transform[3]) / geo_transform[5]) as isize;

    (pixel, line)
}

/// Sample a single pixel value from a raster band
///
/// # Arguments
/// * `band` - The raster band to sample from
/// * `pixel` - X coordinate in pixels
/// * `line` - Y coordinate in pixels
///
/// # Returns
/// The pixel value as a u8 (LCZ code)
pub fn sample_raster_value(band: &RasterBand, pixel: isize, line: isize) -> Result<u8> {
    // Get raster dimensions
    let (raster_width, raster_height) = band.size();

    // Check bounds
    if pixel < 0 || line < 0 || pixel >= raster_width as isize || line >= raster_height as isize {
        return Err(ClassifierError::RasterSampling {
            pixel,
            line,
            message: format!(
                "Coordinates out of bounds. Raster size: {}x{}, requested: ({}, {})",
                raster_width, raster_height, pixel, line
            ),
        });
    }

    // Read single pixel value
    let mut buffer: [u8; 1] = [0];
    band.read_into_slice((pixel, line), (1, 1), (1, 1), &mut buffer, None)
        .map_err(|e| ClassifierError::RasterSampling {
            pixel,
            line,
            message: format!("Failed to read raster value: {}", e),
        })?;

    Ok(buffer[0])
}

/// Validate that a geotransform array contains reasonable values
///
/// Checks for:
/// - Non-zero pixel dimensions
/// - Finite (not NaN or infinite) values
pub fn validate_geo_transform(geo_transform: &[f64; 6]) -> Result<()> {
    // Check that pixel width and height are non-zero
    if geo_transform[1] == 0.0 || geo_transform[5] == 0.0 {
        return Err(ClassifierError::GdalError {
            message: "Invalid geotransform: zero pixel size".to_string(),
        });
    }

    // Check for reasonable values (not NaN or infinite)
    for (i, &value) in geo_transform.iter().enumerate() {
        if !value.is_finite() {
            return Err(ClassifierError::GdalError {
                message: format!(
                    "Invalid geotransform: non-finite value at index {}: {}",
                    i, value
                ),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test validation of WGS84 coordinate bounds
    #[test]
    fn test_validate_coordinates() {
        // Valid coordinates
        assert!((-180.0..=180.0).contains(&0.0));
        assert!((-90.0..=90.0).contains(&45.0));

        // Invalid coordinates
        assert!(!(-180.0..=180.0).contains(&-181.0));
        assert!(!(-90.0..=90.0).contains(&91.0));
    }

    /// Test conversion from geographic coordinates to pixel coordinates
    #[test]
    fn test_geo_to_pixel() {
        // Standard geotransform: origin at (100, 200), 1-degree pixels
        let geo_transform = [100.0, 1.0, 0.0, 200.0, 0.0, -1.0];

        // Test coordinate at origin
        let (pixel, line) = geo_to_pixel(100.0, 200.0, &geo_transform);
        assert_eq!(pixel, 0);
        assert_eq!(line, 0);

        // Test coordinate at (105, 195) - should be pixel (5, 5)
        let (pixel, line) = geo_to_pixel(105.0, 195.0, &geo_transform);
        assert_eq!(pixel, 5);
        assert_eq!(line, 5);
    }

    /// Test validation of geotransform arrays
    #[test]
    fn test_validate_geo_transform() {
        // Valid transform
        let valid_transform = [100.0, 1.0, 0.0, 200.0, 0.0, -1.0];
        assert!(validate_geo_transform(&valid_transform).is_ok());

        // Invalid transform with zero pixel width
        let invalid_transform = [100.0, 0.0, 0.0, 200.0, 0.0, -1.0];
        assert!(validate_geo_transform(&invalid_transform).is_err());

        // Invalid transform with NaN
        let nan_transform = [100.0, f64::NAN, 0.0, 200.0, 0.0, -1.0];
        assert!(validate_geo_transform(&nan_transform).is_err());
    }
}
