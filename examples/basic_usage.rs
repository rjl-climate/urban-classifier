use polars::prelude::*;
use std::collections::HashMap;
use urban_classifier::{Lcz, LczCategory};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Urban Classifier - Basic Usage Example");
    println!("=====================================\n");

    // Example 1: Working with LCZ enums
    println!("1. LCZ Classification System");
    println!("----------------------------");

    // Display all LCZ types
    for code in 1..=17 {
        let lcz = Lcz::from_code(code);
        println!(
            "LCZ {}: {} ({})",
            code,
            lcz.full_name(),
            lcz.simple_category().as_ref()
        );
    }

    println!("\n2. LCZ Categories");
    println!("-----------------");

    // Show category breakdown
    let mut urban_count = 0;
    let mut suburban_count = 0;
    let mut rural_count = 0;

    for code in 1..=17 {
        match Lcz::from_code(code).simple_category() {
            LczCategory::Urban => urban_count += 1,
            LczCategory::Suburban => suburban_count += 1,
            LczCategory::Rural => rural_count += 1,
        }
    }

    println!("Urban types: {}", urban_count);
    println!("Suburban types: {}", suburban_count);
    println!("Rural types: {}", rural_count);

    // Example 2: Creating sample station data
    println!("\n3. Sample Station Data");
    println!("----------------------");

    let stations_df = df! {
        "station_id" => ["LONDON_001", "PARIS_002", "BERLIN_003", "MADRID_004", "ROME_005"],
        "longitude" => [-0.1278, 2.3522, 13.4050, -3.7038, 12.4964],
        "latitude" => [51.5074, 48.8566, 52.5200, 40.4168, 41.9028],
        "elevation" => [35.0, 35.0, 34.0, 650.0, 21.0],
        "country" => ["UK", "France", "Germany", "Spain", "Italy"],
    }?;

    println!("Sample DataFrame:");
    println!("{}", stations_df);

    // Example 3: Manual overrides
    println!("\n4. Manual Override Example");
    println!("--------------------------");

    let mut overrides = HashMap::new();
    overrides.insert("LONDON_001".to_string(), 2); // Force to Compact midrise
    overrides.insert("PARIS_002".to_string(), 11); // Force to Dense trees

    println!("Overrides configured:");
    for (station_id, lcz_code) in &overrides {
        let lcz = Lcz::from_code(*lcz_code);
        println!("  {}: {} ({})", station_id, lcz.full_name(), lcz_code);
    }

    // Example 4: Simulating classification results
    println!("\n5. Simulated Classification Results");
    println!("-----------------------------------");

    // Since we don't have a real GeoTIFF file in this example,
    // we'll simulate what the results would look like
    let simulated_lcz_codes = [1u8, 2, 3, 11, 17];
    let station_ids = [
        "LONDON_001",
        "PARIS_002",
        "BERLIN_003",
        "MADRID_004",
        "ROME_005",
    ];

    println!("Station ID        | LCZ Code | LCZ Name              | Category");
    println!("------------------|----------|----------------------|----------");

    for (station_id, &lcz_code) in station_ids.iter().zip(simulated_lcz_codes.iter()) {
        let lcz = Lcz::from_code(lcz_code);
        println!(
            "{:17} | {:8} | {:20} | {}",
            station_id,
            lcz_code,
            lcz.full_name(),
            lcz.simple_category().as_ref()
        );
    }

    // Example 5: Adding LCZ columns to DataFrame
    println!("\n6. Enhanced DataFrame with LCZ Data");
    println!("------------------------------------");

    // Create LCZ columns
    let lcz_codes_u32: Vec<u32> = simulated_lcz_codes.iter().map(|&x| x as u32).collect();
    let lcz_code_series = Series::new("lcz_code", lcz_codes_u32);
    let lcz_names: Vec<String> = simulated_lcz_codes
        .iter()
        .map(|&code| Lcz::from_code(code).full_name().to_string())
        .collect();
    let lcz_name_series = Series::new("lcz_name", lcz_names);

    let simple_classes: Vec<String> = simulated_lcz_codes
        .iter()
        .map(|&code| Lcz::from_code(code).simple_category().as_ref().to_string())
        .collect();
    let simple_class_series = Series::new("simple_class", simple_classes);

    // Add columns to DataFrame one by one (Polars 0.35 API)
    let mut enhanced_df = stations_df;
    enhanced_df = enhanced_df.with_column(lcz_code_series)?.clone();
    enhanced_df = enhanced_df.with_column(lcz_name_series)?.clone();
    enhanced_df = enhanced_df.with_column(simple_class_series)?.clone();

    println!("{}", enhanced_df);

    println!("\n7. Real Classification Usage");
    println!("----------------------------");
    println!("To perform real classification with WUDAPT data:");
    println!("```rust");
    println!("// Download WUDAPT GeoTIFF from: https://lcz-generator.rub.de/downloads");
    println!("let classifier = UrbanClassifier::new(\"path/to/wudapt_lcz.tif\")?;");
    println!("let result_df = classifier.run_classification(");
    println!("    &stations_df,");
    println!("    \"station_id\",");
    println!("    \"longitude\",");
    println!("    \"latitude\",");
    println!("    Some(&overrides),  // Optional manual overrides");
    println!(")?;");
    println!("```");

    Ok(())
}
