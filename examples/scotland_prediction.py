#!/usr/bin/env python3
"""
Scotland Coordinate Prediction Demo

Based on the coordinates (57.165, -3.23) in Scotland, this script provides
an educated prediction of what the LCZ classification would likely be.
"""

import polars as pl
import urban_classifier


def predict_scotland_classification():
    """
    Predict the likely LCZ classification for coordinates in Scotland.

    Coordinates: 57.165°N, -3.23°W
    This is in the Scottish Highlands region, likely rural/natural area.
    """

    print("Scotland LCZ Classification Prediction")
    print("=====================================")
    print()
    print("Coordinates: 57.165°N, 3.23°W")
    print("Region: Scottish Highlands")
    print()

    # Get LCZ information
    _ = urban_classifier.PyUrbanClassifier.get_lcz_info()

    # Based on Scottish Highland geography, most likely classifications:
    print("GEOGRAPHIC CONTEXT:")
    print("- Scottish Highlands are predominantly rural/natural")
    print("- Landscape: mountains, moorland, forests, lochs")
    print("- Very low population density")
    print("- Minimal urban development")
    print()

    print("MOST LIKELY LCZ CLASSIFICATIONS:")
    print("-" * 40)

    # Most probable classifications for Scottish Highlands
    likely_lcz = [
        (11, "Dense trees", "Rural", 30, "If near forested areas or managed woodland"),
        (12, "Scattered trees", "Rural", 25, "Mixed woodland/moorland landscape"),
        (13, "Bush, scrub", "Rural", 20, "Heather moorland, low shrubs"),
        (14, "Low plants", "Rural", 15, "Open grassland, agricultural areas"),
        (17, "Water", "Rural", 5, "If near a loch (lake) or river"),
        (15, "Bare rock or paved", "Rural", 3, "Rocky outcrops, mountain areas"),
        (
            9,
            "Sparsely built",
            "Suburban",
            2,
            "Very unlikely - only if near small settlement",
        ),
    ]

    total_prob = 0
    for lcz_code, name, category, prob, description in likely_lcz:
        print(
            f"LCZ {lcz_code:2d}: {name:<18} ({category:<8}) - {prob:2d}% | {description}"
        )
        total_prob += prob

    print(f"\nTotal probability for rural/natural: {total_prob}%")
    print()

    # Most likely prediction
    print("MOST LIKELY PREDICTION:")
    print("=" * 25)
    print("LCZ Code: 11, 12, or 13")
    print("LCZ Name: Dense trees, Scattered trees, or Bush/scrub")
    print("Category: Rural")
    print()
    print("REASONING:")
    print("- Scottish Highlands are >95% rural/natural landscape")
    print("- The specific area likely has mixed vegetation")
    print("- Could be managed forest, natural woodland, or moorland")
    print("- Very low probability of any urban/suburban classification")
    print()

    # Create a simulated result
    simulated_result = pl.DataFrame(
        {
            "station_id": ["SCOTLAND_TEST"],
            "longitude": [-3.23],
            "latitude": [57.165],
            "description": ["Scottish Highlands test point"],
            "lcz_code": [12],  # Scattered trees - most likely
            "lcz_name": ["Scattered trees"],
            "simple_class": ["Rural"],
            "confidence": ["High - typical Highland landscape"],
            "notes": ["Predicted based on regional geography"],
        }
    )

    print("SIMULATED CLASSIFICATION RESULT:")
    print("-" * 35)
    print(simulated_result)
    print()

    print("CLIMATE IMPLICATIONS:")
    print("- Minimal urban heat island effect")
    print("- Temperature follows natural elevation/latitude patterns")
    print("- Local climate influenced by topography and vegetation")
    print("- Suitable for 'rural' baseline in climate studies")
    print()

    print("FOR ACTUAL CLASSIFICATION:")
    print("- Download WUDAPT data from: https://lcz-generator.rub.de/downloads")
    print("- Use the urban_classifier.PyUrbanClassifier with real data")
    print("- This will give you the definitive LCZ classification")


if __name__ == "__main__":
    predict_scotland_classification()
