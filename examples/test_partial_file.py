#!/usr/bin/env python3
import polars as pl
import urban_classifier

# Test with the partial file anyway
df = pl.DataFrame({
    "station_id": ["HEATHROW_LHR"],
    "longitude": [-0.4543],
    "latitude": [51.47]
})

print("Attempting classification with partial file...")
try:
    classifier = urban_classifier.PyUrbanClassifier("wudapt_lcz_global.tif")
    result = classifier.run_classification(df, "station_id", "longitude", "latitude")
    print("SUCCESS!")
    print(result)
except Exception as e:
    print(f"FAILED: {e}")