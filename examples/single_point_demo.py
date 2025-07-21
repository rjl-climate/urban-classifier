#!/usr/bin/env python3
"""
Single Point Classification Demo

This script demonstrates how to classify a single geographic coordinate
using the urban_classifier package.
"""

import polars as pl
import sys
from pathlib import Path

try:
    import urban_classifier
except ImportError:
    print("Error: urban_classifier package not found.")
    print("Make sure you've built the package with: maturin develop --features python")
    sys.exit(1)


def classify_single_point(lon, lat, station_id="DEMO_STATION"):
    """
    Classify a single geographic coordinate.

    Args:
        lon: Longitude in decimal degrees
        lat: Latitude in decimal degrees
        station_id: Optional station identifier

    Returns:
        Polars DataFrame with classification results
    """
    print("Urban Classifier - Single Point Demo")
    print("===================================")
    print(f"Coordinates: ({lat}, {lon})")
    print(f"Station ID: {station_id}")
    print()

    # Create a DataFrame with the single point
    df = pl.DataFrame(
        {
            "station_id": [station_id],
            "longitude": [lon],
            "latitude": [lat],
            "description": [f"Test point at {lat}, {lon}"],
        }
    )

    print("Input DataFrame:")
    print(df)
    print()

    # Look for WUDAPT file in common locations
    wudapt_paths = [
        "wudapt_lcz_global.tif",
        "data/wudapt_lcz_global.tif",
        "../data/wudapt_lcz_global.tif",
        "tests/fixtures/sample_wudapt.tif",
        "/tmp/wudapt_lcz_global.tif",
    ]

    wudapt_file = None
    for path in wudapt_paths:
        if Path(path).exists():
            wudapt_file = path
            break

    if wudapt_file:
        print(f"Found WUDAPT file: {wudapt_file}")
        print("Running classification...")

        try:
            # Initialize classifier
            classifier = urban_classifier.PyUrbanClassifier(wudapt_file)

            # Perform classification
            result_df = classifier.run_classification(
                df,
                station_id_col="station_id",
                lon_col="longitude",
                lat_col="latitude",
                overrides=None,
            )

            print("\nClassification Results:")
            print("=" * 50)
            print(result_df)

            # Extract the results
            lcz_code = result_df["lcz_code"][0]
            lcz_name = result_df["lcz_name"][0]
            simple_class = result_df["simple_class"][0]

            print(f"\nSUMMARY for {station_id}:")
            print(f"  Location: {lat}°N, {lon}°E")
            print(f"  LCZ Code: {lcz_code}")
            print(f"  LCZ Name: {lcz_name}")
            print(f"  Category: {simple_class}")

            # Provide interpretation
            print("\nINTERPRETATION:")
            if simple_class == "Urban":
                print("  This location is in an URBAN environment.")
                print("  Expect higher temperatures due to urban heat island effects.")
            elif simple_class == "Suburban":
                print("  This location is in a SUBURBAN environment.")
                print("  Moderate urban influence on local climate.")
            else:
                print("  This location is in a RURAL/NATURAL environment.")
                print("  Minimal urban heat island influence expected.")

            return result_df

        except Exception as e:
            print(f"Classification failed: {e}")
            print("This could be due to:")
            print("- Coordinates outside the WUDAPT coverage area")
            print("- Invalid GeoTIFF file")
            print("- GDAL library issues")
            return None
    else:
        print("No WUDAPT GeoTIFF file found.")
        print("\nTo run actual classification:")
        print("1. Download WUDAPT global LCZ map from:")
        print("   https://lcz-generator.rub.de/downloads")
        print("2. Place the file as 'wudapt_lcz_global.tif' in current directory")
        print("3. Run this script again")

        # Show what the classification system looks like
        print("\nLCZ CLASSIFICATION SYSTEM:")
        print("-" * 30)

        lcz_info = urban_classifier.PyUrbanClassifier.get_lcz_info()
        for i, (code, name, category) in enumerate(
            zip(
                lcz_info["codes"][:10],
                lcz_info["names"][:10],
                lcz_info["categories"][:10],
            )
        ):
            print(f"  LCZ {code}: {name} ({category})")
        print(f"  ... and {len(lcz_info['codes']) - 10} more")

        return None


def main():
    # Test the specific coordinates you provided
    lon = -3.23  # Longitude (West is negative)
    lat = 57.165  # Latitude (North is positive)

    print("This appears to be coordinates in Scotland.")
    print("(Possibly near Aberdeen or in the Highlands)")
    print()

    result = classify_single_point(lon, lat, "SCOTLAND_TEST")

    if result is not None:
        print("\n" + "=" * 60)
        print("CLASSIFICATION COMPLETE!")
        print("You can now use this result for climate analysis.")
    else:
        print("\n" + "=" * 60)
        print("DEMO COMPLETE!")
        print("Download the WUDAPT data to see actual classification.")


if __name__ == "__main__":
    main()
