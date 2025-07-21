use crate::error::{ClassifierError, Result};
use gdal::spatial_ref::{CoordTransform, SpatialRef};
use gdal::raster::RasterBand;

pub fn create_wgs84_to_raster_transform(raster_srs: &SpatialRef) -> Result<CoordTransform> {
    let wgs84 = SpatialRef::from_epsg(4326)?;
    
    CoordTransform::new(&wgs84, raster_srs)
        .map_err(|e| ClassifierError::CoordinateTransform {
            message: format!("Failed to create coordinate transform: {}", e),
        })
}

pub fn transform_coordinate(
    lon: f64,
    lat: f64,
    transform: &CoordTransform,
) -> Result<(f64, f64)> {
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

pub fn geo_to_pixel(x: f64, y: f64, geo_transform: &[f64; 6]) -> (isize, isize) {
    // Apply inverse affine transformation
    // geo_transform: [x_origin, pixel_width, x_skew, y_origin, y_skew, pixel_height]
    // where pixel_height is typically negative
    
    let pixel = ((x - geo_transform[0]) / geo_transform[1]) as isize;
    let line = ((y - geo_transform[3]) / geo_transform[5]) as isize;
    
    (pixel, line)
}

pub fn sample_raster_value(
    band: &RasterBand,
    pixel: isize,
    line: isize,
) -> Result<u8> {
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
    band.read_into_slice(
        (pixel, line),
        (1, 1),
        (1, 1),
        &mut buffer,
        None,
    ).map_err(|e| ClassifierError::RasterSampling {
        pixel,
        line,
        message: format!("Failed to read raster value: {}", e),
    })?;

    Ok(buffer[0])
}

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
                message: format!("Invalid geotransform: non-finite value at index {}: {}", i, value),
            });
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_coordinates() {
        // Valid coordinates
        assert!((-180.0..=180.0).contains(&0.0));
        assert!((-90.0..=90.0).contains(&45.0));
        
        // Invalid coordinates
        assert!(!(-180.0..=180.0).contains(&-181.0));
        assert!(!(-90.0..=90.0).contains(&91.0));
    }

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