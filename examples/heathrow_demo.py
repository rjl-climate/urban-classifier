#!/usr/bin/env python3
"""
Heathrow Airport LCZ Classification Demo

This script demonstrates classifying Heathrow Airport using the urban_classifier.
Heathrow coordinates: 51.4700° N, 0.4543° W
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


def classify_heathrow():
    """
    Classify Heathrow Airport using the Global LCZ Map.

    Heathrow Airport coordinates:
    - Latitude: 51.4700° N
    - Longitude: 0.4543° W (which is -0.4543° in decimal degrees)
    """
    print("🛬 Heathrow Airport LCZ Classification")
    print("=====================================")
    print()

    # Heathrow Airport coordinates
    heathrow_lat = 51.4700
    heathrow_lon = -0.4543  # West longitude is negative

    print("📍 Location: Heathrow Airport, London, UK")
    print(f"🌐 Coordinates: {heathrow_lat}°N, {abs(heathrow_lon)}°W")
    print(f"🔢 Decimal: ({heathrow_lat}, {heathrow_lon})")
    print()

    # Create DataFrame with Heathrow data
    df = pl.DataFrame(
        {
            "station_id": ["HEATHROW_LHR"],
            "longitude": [heathrow_lon],
            "latitude": [heathrow_lat],
            "description": [
                "Heathrow Airport - London's primary international airport"
            ],
            "type": ["Airport"],
            "elevation_m": [25.0],  # Heathrow is about 25m above sea level
        }
    )

    print("Input DataFrame:")
    print(df)
    print()

    # Look for WUDAPT file
    wudapt_paths = [
        "wudapt_lcz_global.tif",
        "data/wudapt_lcz_global.tif",
        "../data/wudapt_lcz_global.tif",
        "/tmp/wudapt_lcz_global.tif",
        str(Path.home() / ".cache" / "urban_classifier" / "wudapt_lcz_global.tif"),
    ]

    wudapt_file = None
    for path in wudapt_paths:
        if Path(path).exists():
            file_size = Path(path).stat().st_size
            if file_size > 3_000_000_000:  # At least 3GB (should be ~4GB)
                wudapt_file = path
                break
            else:
                print(
                    f"⚠️  Found {path} but it's too small ({file_size/1e9:.1f}GB) - likely incomplete"
                )

    if wudapt_file:
        print(f"✅ Found complete WUDAPT file: {wudapt_file}")
        file_size_gb = Path(wudapt_file).stat().st_size / 1e9
        print(f"📊 File size: {file_size_gb:.1f} GB")
        print()
        print("🔍 Running classification...")

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

            print("🎯 CLASSIFICATION RESULTS:")
            print("=" * 50)
            print(result_df)
            print()

            # Extract and interpret the results
            lcz_code = result_df["lcz_code"][0]
            lcz_name = result_df["lcz_name"][0]
            simple_class = result_df["simple_class"][0]

            print("📋 HEATHROW AIRPORT CLASSIFICATION:")
            print("-" * 35)
            print(f"🏷️  LCZ Code: {lcz_code}")
            print(f"📝 LCZ Name: {lcz_name}")
            print(f"🏙️  Category: {simple_class}")
            print()

            # Provide interpretation
            print("🧠 INTERPRETATION:")
            print("-" * 17)

            if lcz_code == 8:
                print("✈️  Large low-rise (LCZ 8) - This is EXPECTED for airports!")
                print("   • Airports typically classified as 'Large low-rise'")
                print(
                    "   • Large buildings (terminals) with extensive open areas (runways)"
                )
                print("   • Low building density but large building footprints")

            elif lcz_code == 9:
                print("🏘️  Sparsely built (LCZ 9) - Also common for airports")
                print("   • Few buildings with extensive open paved areas")
                print("   • Runways and taxiways create large impervious surfaces")

            elif lcz_code == 15:
                print("🛣️  Bare rock or paved (LCZ 15) - Airport infrastructure")
                print("   • Dominated by paved surfaces (runways, taxiways, aprons)")
                print("   • Minimal vegetation or buildings in the classification area")

            elif simple_class == "Urban":
                print(f"🏙️  Urban classification: {lcz_name}")
                print("   • Airport is in or adjacent to urban development")
                print("   • Likely influenced by surrounding built environment")

            elif simple_class == "Suburban":
                print(f"🏘️  Suburban classification: {lcz_name}")
                print("   • Airport has suburban characteristics")
                print("   • Mixed development with moderate building density")

            else:
                print(f"🌳 Rural classification: {lcz_name}")
                print(
                    "   • Unexpected for Heathrow - may indicate data resolution limits"
                )
                print("   • Could be sampling a less developed area near the airport")

            print()
            print("🌡️  CLIMATE IMPLICATIONS:")
            print("-" * 22)

            if simple_class in ["Urban", "Suburban"]:
                print("• Moderate to significant urban heat island effect")
                print("• Airport infrastructure contributes to local warming")
                print("• Large paved areas absorb and re-radiate heat")
                print("• Reduced evapotranspiration compared to natural surfaces")
            else:
                print("• Minimal urban heat island effect")
                print("• Temperature patterns influenced by land surface properties")

            print()
            print("📚 AIRPORT CONTEXT:")
            print("-" * 17)
            print("• Heathrow is one of the world's busiest airports")
            print("• Massive infrastructure: 2 runways, 5 terminals")
            print("• Located in Greater London urban area")
            print("• Airport operations can influence local microclimate")

            return result_df

        except Exception as e:
            print(f"❌ Classification failed: {e}")
            print()
            print("🔧 Possible issues:")
            print("• File may be corrupted or incomplete")
            print("• GDAL library issues")
            print("• Coordinate transformation problems")
            print()
            print("💡 Try re-downloading the WUDAPT data:")
            print("   ./target/release/download_wudapt --force")
            return None
    else:
        print("❌ No complete WUDAPT GeoTIFF file found.")
        print()
        print("📥 To download the Global LCZ Map:")
        print("   ./target/release/download_wudapt")
        print()
        print("📊 Expected file size: ~4GB")
        print("⏱️  Download time: 10-60 minutes")
        print()

        # Show what we expect for Heathrow
        print("🔮 EXPECTED CLASSIFICATION FOR HEATHROW:")
        print("-" * 42)
        print("Most likely: LCZ 8 (Large low-rise) or LCZ 9 (Sparsely built)")
        print("Category: Suburban")
        print("Reasoning:")
        print("• Airports have large buildings (terminals) with low density")
        print("• Extensive paved areas (runways, taxiways)")
        print("• Mix of built and open impervious surfaces")
        print("• Heathrow's massive scale fits 'Large low-rise' pattern")

        return None


def main():
    print("This will classify Heathrow Airport's Local Climate Zone")
    print("using the WUDAPT Global LCZ Map.")
    print()

    result = classify_heathrow()

    if result is not None:
        print()
        print("🎉 CLASSIFICATION COMPLETE!")
        print("=" * 30)
        print("You now have the official LCZ classification for Heathrow Airport")
        print("based on the WUDAPT Global Map derived from satellite imagery.")
        print()
        print("This data can be used for:")
        print("• Climate impact studies")
        print("• Urban heat island research")
        print("• Airport environmental planning")
        print("• Weather and climate modeling")


if __name__ == "__main__":
    main()
