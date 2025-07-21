#!/usr/bin/env python3
import polars as pl
import urban_classifier

# Test European locations only
df = pl.DataFrame({
    "station_id": ["HEATHROW_LHR", "LONDON_CENTER", "PARIS_CENTER"],
    "longitude": [-0.4543, -0.1276, 2.3522],
    "latitude": [51.47, 51.5074, 48.8566]
})

print("Testing European locations with the downloaded WUDAPT file...")
print("File size:", 1.6, "GB")
try:
    classifier = urban_classifier.PyUrbanClassifier("wudapt_lcz_global.tif")
    result = classifier.run_classification(df, "station_id", "longitude", "latitude")
    print("\nACTUAL CLASSIFICATION RESULTS:")
    print(result)
    
    print("\nHeathrow Airport Classification:")
    heathrow = result.filter(pl.col("station_id") == "HEATHROW_LHR")
    lcz_code = heathrow["lcz_code"][0]
    lcz_name = heathrow["lcz_name"][0]
    simple_class = heathrow["simple_class"][0]
    
    print(f"LCZ Code: {lcz_code}")
    print(f"LCZ Name: {lcz_name}")
    print(f"Category: {simple_class}")
    
except Exception as e:
    print(f"FAILED: {e}")