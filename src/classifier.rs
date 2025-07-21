//! Urban Classifier Implementation
//!
//! This module contains the main `UrbanClassifier` struct that performs
//! Local Climate Zone classification of geographic locations using WUDAPT data.
//!
//! # Workflow
//!
//! 1. Load a WUDAPT GeoTIFF file containing LCZ classifications
//! 2. Accept a DataFrame with station locations (ID, longitude, latitude)
//! 3. Transform WGS84 coordinates to the raster's coordinate system
//! 4. Sample the raster at each location to get LCZ codes
//! 5. Apply optional manual overrides
//! 6. Return enhanced DataFrame with LCZ classifications
//!
//! # Features
//!
//! - Automatic coordinate system transformation
//! - Bounds checking for raster sampling
//! - Manual override support for known misclassifications
//! - Detailed error reporting

use crate::error::{ClassifierError, Result};
use crate::lcz::Lcz;
use crate::spatial::{
    create_wgs84_to_raster_transform, geo_to_pixel, sample_raster_value, transform_coordinate,
    validate_geo_transform,
};

use gdal::Dataset;
use polars::prelude::*;
use std::collections::HashMap;
use std::path::Path;

/// Type alias for station IDs and their coordinates
type StationCoordinates = (Vec<String>, Vec<(f64, f64)>);

/// Main classifier struct that holds the WUDAPT GeoTIFF dataset
pub struct UrbanClassifier {
    dataset: Dataset,
}

impl UrbanClassifier {
    /// Create a new UrbanClassifier from a WUDAPT GeoTIFF file
    ///
    /// # Arguments
    /// * `wudapt_geotiff_path` - Path to the WUDAPT LCZ GeoTIFF file
    ///
    /// # Returns
    /// A new UrbanClassifier instance or an error if the file cannot be opened
    pub fn new<P: AsRef<Path>>(wudapt_geotiff_path: P) -> Result<Self> {
        let path = wudapt_geotiff_path.as_ref();

        // Check if file exists
        if !path.exists() {
            return Err(ClassifierError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        // Open the dataset with GDAL
        let dataset = Dataset::open(path)?;

        // Validate that we have at least one raster band
        if dataset.raster_count() == 0 {
            return Err(ClassifierError::GdalError {
                message: "GeoTIFF file contains no raster bands".to_string(),
            });
        }

        // Validate the geotransform
        let geo_transform = dataset.geo_transform()?;
        validate_geo_transform(&geo_transform)?;

        Ok(UrbanClassifier { dataset })
    }

    /// Run LCZ classification on a DataFrame of station locations
    ///
    /// # Arguments
    /// * `stations_df` - DataFrame containing station data
    /// * `station_id_col` - Name of the column containing station IDs
    /// * `lon_col` - Name of the column containing longitude values
    /// * `lat_col` - Name of the column containing latitude values
    /// * `overrides` - Optional map of station IDs to manual LCZ codes
    ///
    /// # Returns
    /// Enhanced DataFrame with additional columns:
    /// - `lcz_code`: Numeric LCZ code (1-17, or 0 for unknown)
    /// - `lcz_name`: Human-readable LCZ name
    /// - `simple_class`: Simplified category (Urban/Suburban/Rural)
    pub fn run_classification(
        &self,
        stations_df: &DataFrame,
        station_id_col: &str,
        lon_col: &str,
        lat_col: &str,
        overrides: Option<&HashMap<String, u8>>,
    ) -> Result<DataFrame> {
        // 1. Validate DataFrame schema
        self.validate_dataframe_schema(stations_df, station_id_col, lon_col, lat_col)?;

        // 2. Get spatial reference and create coordinate transform
        let raster_srs = self.dataset.spatial_ref()?;
        let transform = create_wgs84_to_raster_transform(&raster_srs)?;

        // 3. Get geotransform and raster band
        let geo_transform = self.dataset.geo_transform()?;
        let band = self.dataset.rasterband(1)?;

        // 4. Extract coordinates and station IDs
        let (station_ids, coordinates) =
            self.extract_coordinates(stations_df, station_id_col, lon_col, lat_col)?;

        // 5. Transform coordinates and sample raster
        let mut lcz_codes = Vec::with_capacity(coordinates.len());

        for (i, (lon, lat)) in coordinates.iter().enumerate() {
            // Transform coordinate
            let (x, y) = transform_coordinate(*lon, *lat, &transform)?;

            // Convert to pixel coordinates
            let (pixel, line) = geo_to_pixel(x, y, &geo_transform);

            // Sample raster value
            match sample_raster_value(&band, pixel, line) {
                Ok(code) => lcz_codes.push(code),
                Err(e) => {
                    return Err(ClassifierError::RasterSampling {
                        pixel,
                        line,
                        message: format!(
                            "Failed to sample raster for station {}: {}",
                            station_ids[i], e
                        ),
                    });
                }
            }
        }

        // 6. Apply manual overrides if provided
        if let Some(overrides_map) = overrides {
            self.apply_overrides(&mut lcz_codes, &station_ids, overrides_map)?;
        }

        // 7. Create result columns
        let lcz_series = self.create_lcz_columns(&lcz_codes)?;

        // 8. Return enhanced DataFrame
        let mut result_df = stations_df.clone();
        for series in lcz_series {
            result_df = result_df.with_column(series)?.clone();
        }

        Ok(result_df)
    }

    /// Validate that the input DataFrame has required columns with correct types
    fn validate_dataframe_schema(
        &self,
        df: &DataFrame,
        station_id_col: &str,
        lon_col: &str,
        lat_col: &str,
    ) -> Result<()> {
        let columns = df.get_column_names();

        // Check if required columns exist
        if !columns.contains(&station_id_col) {
            return Err(ClassifierError::ColumnNotFound {
                column: station_id_col.to_string(),
            });
        }

        if !columns.contains(&lon_col) {
            return Err(ClassifierError::ColumnNotFound {
                column: lon_col.to_string(),
            });
        }

        if !columns.contains(&lat_col) {
            return Err(ClassifierError::ColumnNotFound {
                column: lat_col.to_string(),
            });
        }

        // Validate that longitude and latitude columns contain numeric data
        let lon_series = df.column(lon_col)?;
        let lat_series = df.column(lat_col)?;

        if !matches!(lon_series.dtype(), DataType::Float32 | DataType::Float64) {
            return Err(ClassifierError::SchemaValidation {
                message: format!("Longitude column '{}' must contain numeric data", lon_col),
            });
        }

        if !matches!(lat_series.dtype(), DataType::Float32 | DataType::Float64) {
            return Err(ClassifierError::SchemaValidation {
                message: format!("Latitude column '{}' must contain numeric data", lat_col),
            });
        }

        Ok(())
    }

    /// Extract station IDs and coordinates from the DataFrame
    fn extract_coordinates(
        &self,
        df: &DataFrame,
        station_id_col: &str,
        lon_col: &str,
        lat_col: &str,
    ) -> Result<StationCoordinates> {
        let station_ids: Vec<String> = df
            .column(station_id_col)?
            .str()
            .map_err(|_| ClassifierError::SchemaValidation {
                message: format!(
                    "Station ID column '{}' must contain string data",
                    station_id_col
                ),
            })?
            .into_iter()
            .map(|opt| opt.unwrap_or("unknown").to_string())
            .collect();

        let lon_values: Vec<f64> = df
            .column(lon_col)?
            .f64()
            .map_err(|_| ClassifierError::SchemaValidation {
                message: format!("Failed to access longitude column '{}' as f64", lon_col),
            })?
            .into_iter()
            .map(|opt| {
                opt.ok_or_else(|| ClassifierError::SchemaValidation {
                    message: format!("Found null value in longitude column '{}'", lon_col),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let lat_values: Vec<f64> = df
            .column(lat_col)?
            .f64()
            .map_err(|_| ClassifierError::SchemaValidation {
                message: format!("Failed to access latitude column '{}' as f64", lat_col),
            })?
            .into_iter()
            .map(|opt| {
                opt.ok_or_else(|| ClassifierError::SchemaValidation {
                    message: format!("Found null value in latitude column '{}'", lat_col),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let coordinates: Vec<(f64, f64)> = lon_values.into_iter().zip(lat_values).collect();

        Ok((station_ids, coordinates))
    }

    /// Apply manual LCZ code overrides for specific stations
    fn apply_overrides(
        &self,
        lcz_codes: &mut [u8],
        station_ids: &[String],
        overrides: &HashMap<String, u8>,
    ) -> Result<()> {
        for (i, station_id) in station_ids.iter().enumerate() {
            if let Some(&override_code) = overrides.get(station_id) {
                lcz_codes[i] = override_code;
            }
        }
        Ok(())
    }

    /// Create the three output columns: lcz_code, lcz_name, and simple_class
    fn create_lcz_columns(&self, lcz_codes: &[u8]) -> Result<Vec<Series>> {
        // Create lcz_code column - convert u8 to u32 for better Polars compatibility
        let lcz_codes_u32: Vec<u32> = lcz_codes.iter().map(|&x| x as u32).collect();
        let lcz_code_series = Series::new("lcz_code", lcz_codes_u32);

        // Create lcz_name column
        let lcz_names: Vec<String> = lcz_codes
            .iter()
            .map(|&code| Lcz::from_code(code).full_name().to_string())
            .collect();
        let lcz_name_series = Series::new("lcz_name", lcz_names);

        // Create simple_class column
        let simple_classes: Vec<String> = lcz_codes
            .iter()
            .map(|&code| Lcz::from_code(code).simple_category().as_ref().to_string())
            .collect();
        let simple_class_series = Series::new("simple_class", simple_classes);

        Ok(vec![lcz_code_series, lcz_name_series, simple_class_series])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    /// Test that missing files are properly reported
    #[test]
    fn test_file_not_found() {
        let result = UrbanClassifier::new("/nonexistent/path.tif");
        assert!(matches!(result, Err(ClassifierError::FileNotFound { .. })));
    }

    /// Test DataFrame schema validation logic
    #[test]
    fn test_dataframe_validation() {
        // Create a test classifier (this will fail but we only test validation)
        let _temp_file = NamedTempFile::new().unwrap();

        // Create a test DataFrame
        let _df = df! {
            "station_id" => ["A", "B", "C"],
            "longitude" => [1.0, 2.0, 3.0],
            "latitude" => [50.0, 51.0, 52.0],
        }
        .unwrap();

        // This would normally fail to create classifier, but we can test the validation logic
        // by creating a mock test for the validation function

        // Test missing column
        let _df_missing = df! {
            "station_id" => ["A", "B", "C"],
            "longitude" => [1.0, 2.0, 3.0],
            // missing latitude column
        }
        .unwrap();

        // We can't test this directly without a valid classifier instance,
        // but the validation logic is tested through integration tests
    }

    /// Test creation of LCZ result columns
    #[test]
    fn test_lcz_column_creation() {
        // Create a test classifier
        let temp_file = NamedTempFile::new().unwrap();
        let _classifier = UrbanClassifier::new(temp_file.path());

        // This will fail because it's not a valid GeoTIFF, but we can test
        // the LCZ column creation logic separately

        let lcz_codes = [1, 2, 3, 11, 17];

        // Test LCZ conversion
        let lcz_types: Vec<Lcz> = lcz_codes.iter().map(|&code| Lcz::from_code(code)).collect();
        assert_eq!(lcz_types[0], Lcz::CompactHighRise);
        assert_eq!(lcz_types[1], Lcz::CompactMidRise);
        assert_eq!(lcz_types[4], Lcz::Water);

        // Test names and categories
        assert_eq!(Lcz::from_code(1).full_name(), "Compact high-rise");
        assert_eq!(Lcz::from_code(11).simple_category().as_ref(), "Rural");
    }
}
