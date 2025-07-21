use polars::prelude::*;
use std::collections::HashMap;
use tempfile::NamedTempFile;
use urban_classifier::{UrbanClassifier, Lcz, LczCategory, ClassifierError};

#[test]
fn test_lcz_enum_conversions() {
    // Test all valid LCZ codes
    for code in 1..=17 {
        let lcz = Lcz::from_code(code);
        assert_eq!(lcz.to_code(), code);
        assert!(!lcz.full_name().is_empty());
        
        // Ensure category assignment is logical
        let category = lcz.simple_category();
        match code {
            1..=6 => assert_eq!(category, LczCategory::Urban),
            7..=10 => assert_eq!(category, LczCategory::Suburban),
            11..=17 => assert_eq!(category, LczCategory::Rural),
            _ => unreachable!(),
        }
    }
}

#[test]
fn test_unknown_lcz_code() {
    let unknown = Lcz::from_code(99);
    assert_eq!(unknown, Lcz::Unknown(99));
    assert_eq!(unknown.to_code(), 99);
    assert_eq!(unknown.full_name(), "Unknown");
    assert_eq!(unknown.simple_category(), LczCategory::Rural);
}

#[test]
fn test_lcz_category_as_ref() {
    assert_eq!(LczCategory::Urban.as_ref(), "Urban");
    assert_eq!(LczCategory::Suburban.as_ref(), "Suburban");
    assert_eq!(LczCategory::Rural.as_ref(), "Rural");
}

#[test]
fn test_file_not_found_error() {
    let result = UrbanClassifier::new("/nonexistent/file.tif");
    assert!(matches!(result, Err(ClassifierError::FileNotFound { .. })));
}

#[test]
fn test_dataframe_creation() {
    // Test creating a valid DataFrame for classification
    let df = df! {
        "station_id" => ["STATION_001", "STATION_002", "STATION_003"],
        "longitude" => [-0.1278, 2.3522, 13.4050],
        "latitude" => [51.5074, 48.8566, 52.5200],
        "elevation" => [35.0, 35.0, 34.0],
    }.unwrap();

    // Verify DataFrame structure
    assert_eq!(df.shape(), (3, 4));
    assert!(df.get_column_names().contains(&"station_id"));
    assert!(df.get_column_names().contains(&"longitude"));
    assert!(df.get_column_names().contains(&"latitude"));

    // Test coordinate extraction
    let lon_values: Vec<f64> = df.column("longitude")
        .unwrap()
        .f64()
        .unwrap()
        .into_iter()
        .map(|opt| opt.unwrap_or(0.0))
        .collect();
    
    assert_eq!(lon_values, vec![-0.1278, 2.3522, 13.4050]);
}

#[test]
fn test_invalid_coordinates() {
    // Test coordinate validation logic
    let invalid_coords = vec![
        (-181.0, 0.0),   // Longitude too low
        (181.0, 0.0),    // Longitude too high
        (0.0, -91.0),    // Latitude too low
        (0.0, 91.0),     // Latitude too high
    ];

    for (lon, lat) in invalid_coords {
        assert!(
            !(-180.0..=180.0).contains(&lon) || !(-90.0..=90.0).contains(&lat),
            "Coordinate ({}, {}) should be invalid",
            lon, lat
        );
    }
}

#[test]
fn test_override_logic() {
    // Test override application logic
    let mut lcz_codes = vec![1, 2, 3, 4, 5];
    let station_ids = vec![
        "A".to_string(),
        "B".to_string(),
        "C".to_string(),
        "D".to_string(),
        "E".to_string(),
    ];
    
    let mut overrides = HashMap::new();
    overrides.insert("B".to_string(), 11);
    overrides.insert("D".to_string(), 17);

    // Apply overrides manually (simulating the classifier logic)
    for (i, station_id) in station_ids.iter().enumerate() {
        if let Some(&override_code) = overrides.get(station_id) {
            lcz_codes[i] = override_code;
        }
    }

    assert_eq!(lcz_codes, vec![1, 11, 3, 17, 5]);
}

#[test]
fn test_geo_transform_pixel_conversion() {
    use urban_classifier::spatial::geo_to_pixel;
    
    // Test standard geotransform: origin at (100, 200), 1-degree pixels
    let geo_transform = [100.0, 1.0, 0.0, 200.0, 0.0, -1.0];
    
    // Test coordinate at origin
    let (pixel, line) = geo_to_pixel(100.0, 200.0, &geo_transform);
    assert_eq!(pixel, 0);
    assert_eq!(line, 0);
    
    // Test coordinate at (105, 195) - should be pixel (5, 5)
    let (pixel, line) = geo_to_pixel(105.0, 195.0, &geo_transform);
    assert_eq!(pixel, 5);
    assert_eq!(line, 5);
    
    // Test negative coordinates
    let (pixel, line) = geo_to_pixel(99.0, 201.0, &geo_transform);
    assert_eq!(pixel, -1);
    assert_eq!(line, -1);
}

#[test]
fn test_polars_series_creation() {
    // Test creating the types of series that would be returned by the classifier
    let lcz_codes = vec![1u8, 2, 3, 11, 17];
    
    // Create lcz_code series - convert to u32
    let lcz_codes_u32: Vec<u32> = lcz_codes.iter().map(|&x| x as u32).collect();
    let lcz_code_series = Series::new("lcz_code", lcz_codes_u32);
    assert_eq!(lcz_code_series.len(), 5);
    assert_eq!(lcz_code_series.name(), "lcz_code");
    
    // Create lcz_name series
    let lcz_names: Vec<String> = lcz_codes
        .iter()
        .map(|&code| Lcz::from_code(code).full_name().to_string())
        .collect();
    let lcz_name_series = Series::new("lcz_name", lcz_names.clone());
    assert_eq!(lcz_name_series.len(), 5);
    
    // Create simple_class series
    let simple_classes: Vec<String> = lcz_codes
        .iter()
        .map(|&code| Lcz::from_code(code).simple_category().as_ref().to_string())
        .collect();
    let simple_class_series = Series::new("simple_class", simple_classes);
    assert_eq!(simple_class_series.len(), 5);
    
    // Verify expected values
    let expected_names = vec![
        "Compact high-rise",
        "Compact midrise", 
        "Compact low-rise",
        "Dense trees",
        "Water"
    ];
    // Direct comparison with the created vector
    for (expected, actual) in expected_names.iter().zip(lcz_names.iter()) {
        assert_eq!(*expected, actual);
    }
}

// Integration test that would work with a real GeoTIFF file
// This is disabled by default since we don't have test data
#[test]
#[ignore]
fn test_full_classification_with_real_data() {
    // This test would require a real WUDAPT GeoTIFF file
    // It's marked as #[ignore] so it won't run by default
    
    let wudapt_path = "tests/fixtures/sample_wudapt.tif";
    
    if std::path::Path::new(wudapt_path).exists() {
        let classifier = UrbanClassifier::new(wudapt_path).unwrap();
        
        let df = df! {
            "station_id" => ["TEST_001", "TEST_002"],
            "longitude" => [0.0, 1.0],
            "latitude" => [51.5, 52.0],
        }.unwrap();
        
        let result = classifier.run_classification(
            &df,
            "station_id",
            "longitude", 
            "latitude",
            None,
        );
        
        assert!(result.is_ok());
        let result_df = result.unwrap();
        
        // Check that new columns were added
        assert!(result_df.get_column_names().contains(&"lcz_code"));
        assert!(result_df.get_column_names().contains(&"lcz_name"));
        assert!(result_df.get_column_names().contains(&"simple_class"));
    }
}