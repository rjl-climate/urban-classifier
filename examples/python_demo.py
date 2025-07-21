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
        for i, (code, name, category) in enumerate(
            zip(lcz_info["codes"], lcz_info["names"], lcz_info["categories"])
        ):
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

    # Create sample DataFrame with European cities and Heathrow Airport
    stations_data = {
        "station_id": [
            "LONDON_001",
            "PARIS_002",
            "BERLIN_003",
            "MADRID_004",
            "ROME_005",
            "HEATHROW",
        ],
        "longitude": [-0.1278, 2.3522, 13.4050, -3.7038, 12.4964, -0.454295],
        "latitude": [51.5074, 48.8566, 52.5200, 40.4168, 41.9028, 51.470020],
        "elevation": [35.0, 35.0, 34.0, 650.0, 21.0, 25.0],
        "country": ["UK", "France", "Germany", "Spain", "Italy", "UK"],
    }

    df = pl.DataFrame(stations_data)
    print("Sample DataFrame:")
    print(df)

    # Example 3: Demonstrate manual overrides
    print("\n3. Manual Override Configuration")
    print("--------------------------------")

    # Create manual overrides for some stations
    overrides = {
        "LONDON_001": 2,  # Force to Compact midrise
        "PARIS_002": 11,  # Force to Dense trees
    }

    print("Manual overrides configured:")
    for station_id, lcz_code in overrides.items():
        print(f"  {station_id}: LCZ {lcz_code}")

    # Example 4: Classification (requires real WUDAPT file)
    print("\n4. Classification Example")
    print("-------------------------")

    # Use the real WUDAPT file
    wudapt_file = (
        "/Users/richardlyon/dev/mine/rust/urban-classifier/wudapt_lcz_global.tif"
    )

    if Path(wudapt_file).exists():
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
                overrides=overrides,
            )

            print("\nClassification Results:")
            print(result_df)

            # Analyze results
            print("\n5. Results Analysis")
            print("-------------------")

            # Count by category
            category_counts = result_df.group_by("simple_class").agg(
                pl.count().alias("count")
            )
            print("\nStations by category:")
            print(category_counts)

            # Show override effects
            print("\nStations with manual overrides:")
            override_results = result_df.filter(
                pl.col("station_id").is_in(list(overrides.keys()))
            )
            print(
                override_results.select(
                    ["station_id", "lcz_code", "lcz_name", "simple_class"]
                )
            )

        except Exception as e:
            print(f"Classification failed: {e}")
            print("Error details:", str(e))
            return
    else:
        print(f"Error: WUDAPT file not found at {wudapt_file}")
        print("Please ensure the file exists at the specified location.")
        return

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
