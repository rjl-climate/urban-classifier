#!/usr/bin/env python3
import polars as pl
import urban_classifier

# Test multiple well-known locations
df = pl.DataFrame(
    {
        "station_id": [
            "HEATHROW_LHR",
            "LONDON_CENTER",
            "PARIS_CENTER",
            "NYC_MANHATTAN",
        ],
        "longitude": [-0.4543, -0.1276, 2.3522, -73.9857],
        "latitude": [51.47, 51.5074, 48.8566, 40.7484],
    }
)

print("Testing multiple locations with the downloaded WUDAPT file...")
try:
    classifier = urban_classifier.PyUrbanClassifier("wudapt_lcz_global.tif")
    result = classifier.run_classification(df, "station_id", "longitude", "latitude")
    print("RESULTS:")
    print(result)
except Exception as e:
    print(f"FAILED: {e}")
