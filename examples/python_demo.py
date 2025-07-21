#!/usr/bin/env python3
"""
Urban Classifier Python Demo

This script demonstrates how to use the urban_classifier Python package
to classify geographic coordinates using the Local Climate Zone (LCZ) system.

Requirements:
- urban_classifier package (built with maturin)
- polars
- WUDAPT GeoTIFF file (download from https://lcz-generator.rub.de/downloads)
"""

import polars as pl
import sys
from pathlib import Path

try:
    import urban_classifier
except ImportError:
    print("Error: urban_classifier package not found.")
    print("Please build and install the package first:")
    print("  maturin develop --features python")
    sys.exit(1)


def main():
    print("Urban Classifier Python Demo")
    print("============================\n")

    # Example 1: Get LCZ information
    print("1. LCZ Classification System Information")
    print("----------------------------------------")
    
    try:
        lcz_info = urban_classifier.PyUrbanClassifier.get_lcz_info()
        
        print("Available LCZ classes:")
        for i, (code, name, category) in enumerate(zip(
            lcz_info["codes"], 
            lcz_info["names"], 
            lcz_info["categories"]
        )):
            print(f"  LCZ {code}: {name} ({category})")
            if i >= 5:  # Limit output for demo
                print(f"  ... and {len(lcz_info['codes']) - 6} more")
                break
                
    except Exception as e:
        print(f"Error getting LCZ info: {e}")
        return

    # Example 2: Create sample station data
    print("\n2. Sample Station Data")
    print("----------------------")
    
    # Create sample DataFrame with European cities
    stations_data = {
        "station_id": ["LONDON_001", "PARIS_002", "BERLIN_003", "MADRID_004", "ROME_005"],
        "longitude": [-0.1278, 2.3522, 13.4050, -3.7038, 12.4964],
        "latitude": [51.5074, 48.8566, 52.5200, 40.4168, 41.9028],
        "elevation": [35.0, 35.0, 34.0, 650.0, 21.0],
        "country": ["UK", "France", "Germany", "Spain", "Italy"],
    }
    
    df = pl.DataFrame(stations_data)
    print("Sample DataFrame:")
    print(df)

    # Example 3: Demonstrate manual overrides
    print("\n3. Manual Override Configuration")
    print("--------------------------------")
    
    # Create manual overrides for some stations
    overrides = {
        "LONDON_001": 2,   # Force to Compact midrise
        "PARIS_002": 11,   # Force to Dense trees
    }
    
    print("Manual overrides configured:")
    for station_id, lcz_code in overrides.items():
        print(f"  {station_id}: LCZ {lcz_code}")

    # Example 4: Classification (requires real WUDAPT file)
    print("\n4. Classification Example")
    print("-------------------------")
    
    # Check for WUDAPT file
    wudapt_paths = [
        "wudapt_lcz_global.tif",
        "data/wudapt_lcz_global.tif",
        "../data/wudapt_lcz_global.tif",
        "tests/fixtures/sample_wudapt.tif",
    ]
    
    wudapt_file = None
    for path in wudapt_paths:
        if Path(path).exists():
            wudapt_file = path
            break
    
    if wudapt_file:
        print(f"Found WUDAPT file: {wudapt_file}")
        try:
            # Initialize classifier
            classifier = urban_classifier.PyUrbanClassifier(wudapt_file)
            
            # Validate DataFrame
            print("Validating DataFrame schema...")
            is_valid = classifier.validate_dataframe(
                df, "station_id", "longitude", "latitude"
            )
            print(f"DataFrame is valid: {is_valid}")
            
            # Perform classification
            print("Running classification...")
            result_df = classifier.run_classification(
                df,
                station_id_col="station_id",
                lon_col="longitude", 
                lat_col="latitude",
                overrides=overrides
            )
            
            print("\nClassification Results:")
            print(result_df)
            
            # Analyze results
            print("\n5. Results Analysis")
            print("-------------------")
            
            # Count by category
            category_counts = result_df.group_by("simple_class").count()
            print("Stations by category:")
            print(category_counts)
            
            # Show override effects
            print("\nStations with manual overrides:")
            override_results = result_df.filter(
                pl.col("station_id").is_in(list(overrides.keys()))
            )
            print(override_results.select(["station_id", "lcz_code", "lcz_name", "simple_class"]))
            
        except Exception as e:
            print(f"Classification failed: {e}")
            print("This is expected if the WUDAPT file is not a valid GeoTIFF")
    else:
        print("No WUDAPT GeoTIFF file found. To run classification:")
        print("1. Download WUDAPT global LCZ map from: https://lcz-generator.rub.de/downloads")
        print("2. Place the file in the current directory as 'wudapt_lcz_global.tif'")
        print("3. Run this script again")
        
        print("\nSimulated classification results:")
        # Add simulated LCZ data for demonstration
        simulated_lcz = pl.DataFrame({
            "station_id": stations_data["station_id"],
            "lcz_code": [1, 2, 3, 11, 17],
            "lcz_name": ["Compact high-rise", "Compact midrise", "Compact low-rise", "Dense trees", "Water"],
            "simple_class": ["Urban", "Urban", "Urban", "Rural", "Rural"]
        })
        
        result_df = df.join(simulated_lcz, on="station_id")
        print(result_df)

    # Example 5: Working with results
    print("\n6. Working with Classification Results")
    print("--------------------------------------")
    
    print("Example operations you can perform:")
    print("- Filter urban stations: df.filter(pl.col('simple_class') == 'Urban')")
    print("- Count by LCZ type: df.group_by('lcz_name').count()")
    print("- Export results: df.write_csv('classified_stations.csv')")
    print("- Geographic analysis: Join with climate data, elevation, etc.")

    print("\n7. Integration with Other Tools")
    print("-------------------------------")
    print("The results can be easily integrated with:")
    print("- GeoPandas for spatial analysis")
    print("- Matplotlib/Plotly for visualization")
    print("- Scikit-learn for machine learning")
    print("- Climate analysis packages")


if __name__ == "__main__":
    main()