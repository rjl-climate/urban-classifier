# Urban classifer

A rust application for generating a database of urban classification information for UK locations.

Principles:

1. Define an analysis zone or buffer around a point. This will be characterised by a radius of 500 metres to 1 kilometre, say.

2. A few potential categories. Land cover and land use. Urban morphology in built form. And anthropogenic factors.

3. Land cover and land use will include features like:
- Impervious surface fraction
- Vegetation fraction
- Water body fraction
- Land use mix

4. Urban morphology will include things like:
- Building height
- Sky view factor
- Building surface area density
- Canyon aspect ratio
This is unlikely to be useful for me.

5. Anthropogenic factors will be things like:
- Population density
- Traffic and road volume density
- Waste heat flux

Probably the population density is the one.

Of these, I can see the population density Which comes from ONS statistics, And land cover statistics.

There's an existing scheme. It's called the Local Climate Zone Scheme.
The LCZ system has 17 standard classes that are highly relevant to weather:
LCZ 1: Compact high-rise
LCZ 2: Compact mid-rise
LCZ 3: Compact low-rise
LCZ 4: Open high-rise
LCZ 5: Open mid-rise
LCZ 8: Large low-rise (e.g., retail parks)
LCZ A: Dense trees
LCZ D: Low plants (e.g., parks, farmland)
LCZ G: Water

Rather than a crude urban/rural classification system, or some algorithm that tries to determine the dominant land cover classification (otherwise known as the buffer method), or quantitative approach that combines all these factors to create continuous numerical features for the buffer zone that surrounds the station: it's best to try and classify a given lat/long coordinate into one of the local climate zone classifications. This is hard and the input to it is the previous quantitative approach.

In fact, we use a pre-computed global LCZ map This is fairly straightforward in python: Just use the WUDAPT Portal's LCZ Global Map page. It has the advantage that it is de facto global standard for creating a publicly accessible, standardized classification of urban forms for climate studies, and so it will withstand anyone who criticises the analysis.


```python
import geopandas as gpd
import pandas as pd
import rasterio
from shapely.geometry import Point

# 1. Load your weather station data
# Assuming you have a CSV with columns: 'station_id', 'latitude', 'longitude'
station_df = pd.read_csv('your_stations.csv')

# Convert the DataFrame to a GeoDataFrame
geometry = [Point(xy) for xy in zip(station_df['longitude'], station_df['latitude'])]
stations_gdf = gpd.GeoDataFrame(station_df, geometry=geometry, crs="EPSG:4326") # WGS84

# 2. Open the downloaded LCZ GeoTIFF file
# Update this path to where you saved the file
lcz_raster_path = 'path/to/your/global_lcz_map.tif'
lcz_raster = rasterio.open(lcz_raster_path)

# 3. Ensure your station coordinates are in the same CRS as the raster
# The global LCZ map uses WGS84 (EPSG:4326), so we are good.
# If they were different, you would use: stations_gdf = stations_gdf.to_crs(lcz_raster.crs)

# 4. Extract the LCZ value for each station point
# The rasterio.sample.sample_gen function is perfect for this
coords = [(p.x, p.y) for p in stations_gdf.geometry]
lcz_values = [val[0] for val in lcz_raster.sample(coords)]

# Add the LCZ values back to your GeoDataFrame
stations_gdf['lcz'] = lcz_values

# 5. Print the results
print(stations_gdf[['station_id', 'latitude', 'longitude', 'lcz']].head())

# You can now map the numeric LCZ codes to their names if you wish
lcz_mapping = {
    1: 'Compact high-rise', 2: 'Compact mid-rise', 3: 'Compact low-rise',
    4: 'Open high-rise', 5: 'Open mid-rise', 6: 'Open low-rise',
    7: 'Lightweight low-rise', 8: 'Large low-rise', 9: 'Sparsely built',
    10: 'Heavy industry', 17: 'Water', # LCZ G is often coded as 17
    # Add other natural LCZ codes (A-F) as needed, usually 11-16
}
stations_gdf['lcz_name'] = stations_gdf['lcz'].map(lcz_mapping)
print("\nWith names:")
print(stations_gdf[['station_id', 'lcz', 'lcz_name']].head())
```

The disadvantage is the resolution is only 100 metres, but that's probably okay for what we're doing.

If that's not high enough, we can actually create our own LCZ map using a fantastic Python library: GitHub Repository: https://github.com/udellgroup/lcz-generator

In this approach, we would:
- Define areas of interest
- Get satellite imagery
- Create training data
- Run the generator, which will then produce the classifications

It's reasonably straightforward to map LCZ categories into simpler classifications like urban and suburban. For example:

LCZ Code	Official LCZ Name	Proposed Category	Justification (Climate & Morphology)
1	Compact high-rise	Urban	The most intensely developed urban form. Extremely high building and impervious surface density. The classic city centre/downtown.
2	Compact mid-rise	Urban	Dense urban fabric, typical of historic European city centres and inner-city residential areas. High impervious fraction.
3	Compact low-rise	Urban	Dense, but lower buildings (e.g., terraced housing). Still a very high fraction of artificial surfaces and minimal vegetation.
8	Large low-rise	Urban	Refers to retail parks, industrial estates, warehouses. Characterized by vast roofs and parking lots. Climatically, these are intensely urban (hot spots).
10	Heavy industry	Urban	Large-scale industrial areas with significant anthropogenic heat emissions and impervious surfaces.
4	Open high-rise	Suburban	Spaced-out high-rise buildings set in green landscapes (e.g., tower-in-the-park developments). A mix of built and vegetated surfaces.
5	Open mid-rise	Suburban	Spaced-out apartment blocks with significant green space. A classic suburban form.
6	Open low-rise	Suburban	The quintessential suburb: detached or semi-detached single-family homes with private gardens and lawns. Abundant vegetation mixed with built surfaces.
7	Lightweight low-rise	Suburban	(Less common in the UK). Represents informal settlements. Structurally it's low-rise and open, fitting best with a suburban profile in a UK context.
9	Sparsely built	Rural	Very low-density development, often on the edge of the countryside. Small settlements, hamlets, or isolated buildings. The landscape character is predominantly natural.
A	Dense trees	Rural	Forests, dense woodlands. A natural landscape.
B	Scattered trees	Rural	Parkland, orchards, open woodland. Predominantly natural.
C	Bush, scrub	Rural	Natural, unmanaged vegetation.
D	Low plants	Rural	The most common rural class in the UK: farmland, pastures, playing fields, large parks.
E	Bare rock or paved	Rural	Refers to natural rock/soil or paved areas in a rural context (e.g., quarries, large farmyards). Use context if needed.
F	Bare soil or sand	Rural	Beaches, ploughed fields. Natural land cover.
G	Water	Rural	Rivers, lakes, reservoirs. Natural land cover.


We actually need two different modules rather than a mega module.

- An urban classifier For broad classification.
- A separate "enrichment" module that establishes for each station a series of small buffers for micro-scale buffering.

---
## Rust urban classifier spec:

Here is the comprehensive specification for the Urban Classifier module.

---

### **Specification for Rust Urban Classifier Crate**

**1. Project Overview & Goal**

You are an expert **Rust developer** specializing in high-performance geospatial data analysis. Your task is to build a Rust crate named `urban_classifier`. This crate will be a core component of a larger analysis library and must be callable from Python.

The primary goal of this module is to classify geographic coordinates according to their urban environment, leveraging Rust's performance and safety. The classification must be robust, scientifically defensible, and based on the standard **Local Climate Zone (LCZ)** system. The crate should expose a simple, well-documented API and allow for manual validation and overrides.

**2. Essential Reading & Contextual Resources**


To understand the domain and produce accurate code, you must first review the concepts and data from the following resources:

*   **The Local Climate Zone (LCZ) Framework:** This is the scientific foundation. Understand what the classes mean.
    *   **Primary Source:** Stewart, I. D., & Oke, T. R. (2012). "Local Climate Zones for Urban Temperature Studies". *Bulletin of the American Meteorological Society*. You can read the abstract and see the class definitions here: [https://journals.ametsoc.org/view/journals/bams/93/12/bams-d-11-00019.1.xml](https://journals.ametsoc.org/view/journals/bams/93/12/bams-d-11-00019.1.xml)
    *   **Visual Guide:** Review the visual guide to the 17 LCZ types here: [http://www.wudapt.org/lcz/](http://www.wudapt.org/lcz/)

*   **The World Urban Database and Access Portal Tools (WUDAPT):** This is the project that provides the data we will use.
    *   **Homepage:** [http://www.wudapt.org/](http://www.wudapt.org/)
    *   **Global LCZ Map Data Source:** The module will use the pre-computed global LCZ map. The data can be downloaded as a GeoTIFF from here. The code you write should assume the user has already downloaded this file. [https://lcz-generator.rub.de/downloads](https://lcz-generator.rub.de/downloads)
    *   **Data Timeliness:** Note that this global map is based on **~2018** satellite imagery.

**3. Core Functional Requirements**

The crate must provide a Rust-native API and Python bindings (`pyo3`) to perform the following:

1.  **Load Station Data:** Accept a **Polars DataFrame** containing station IDs and geographic coordinates (latitude, longitude).
2.  **Classify from WUDAPT:** For each coordinate, sample the global LCZ GeoTIFF file to extract its LCZ code. This process must handle coordinate system transformations.
3.  **Handle Manual Overrides:** Allow the user to provide a `HashMap` of manual corrections to override the WUDAPT classification for specific stations.
4.  **Provide Rich Types:** Use a Rust `enum` to represent LCZ classes in a type-safe way, with methods to provide human-readable names and simplified "Urban/Suburban/Rural" categories.
5.  **Return Enhanced Data:** The final output should be a new Polars DataFrame with columns for `lcz_code`, `lcz_name`, and `simple_class` appended.

**4. Proposed Crate Structure & API**

The crate's functionality will be centered around an `UrbanClassifier` struct and a type-safe `Lcz` enum.

```rust
// --- src/lcz.rs ---
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lcz {
    CompactHighRise, // 1
    CompactMidRise,  // 2
    // ... all 17 classes
    Water,           // 17
    Unknown(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LczCategory {
    Urban,
    Suburban,
    Rural,
}

impl Lcz {
    pub fn from_code(code: u8) -> Self { /* ... */ }
    pub fn to_code(&self) -> u8 { /* ... */ }
    pub fn full_name(&self) -> &'static str { /* ... */ }
    pub fn simple_category(&self) -> LczCategory { /* ... */ }
}


// --- src/classifier.rs ---
use polars::prelude::*;
use gdal::Dataset;
use std::collections::HashMap;
use std::path::Path;
use crate::lcz::Lcz;
use crate::error::ClassifierError;

pub struct UrbanClassifier {
    dataset: Dataset,
}

impl UrbanClassifier {
    /// Creates a new classifier by opening the WUDAPT GeoTIFF dataset.
    pub fn new<P: AsRef<Path>>(wudapt_geotiff_path: P) -> Result<Self, ClassifierError> {
        // ... implementation ...
    }

    /// Classifies points in a DataFrame.
    pub fn run_classification(
        &self,
        stations_df: &DataFrame,
        station_id_col: &str,
        lon_col: &str,
        lat_col: &str,
        overrides: Option<&HashMap<String, u8>>,
    ) -> Result<DataFrame, ClassifierError> {
        // ... implementation ...
    }
}
```

**5. Detailed Method & Type Specifications**

**5.1. `Lcz` Enum (`lcz.rs`)**

*   Define a comprehensive `enum Lcz` for all 17 standard classes, plus a variant for `Unknown(u8)` to handle unexpected values from the raster.
*   Implement `Lcz::from_code(u8) -> Self` to convert a raw integer from the raster into the corresponding enum variant.
*   Implement `Lcz::full_name(&self) -> &'static str` to return the full descriptive name (e.g., "Compact high-rise").
*   Implement `Lcz::simple_category(&self) -> LczCategory` which returns the appropriate `Urban`, `Suburban`, or `Rural` enum.
*   The `LczCategory` enum should implement `AsRef<str>` for easy conversion to a string.

**5.2. `UrbanClassifier::new()` (`classifier.rs`)**

*   **Parameters:** `wudapt_geotiff_path`: A generic path type (`<P: AsRef<Path>>`).
*   **Action:**
    *   Check if the file exists. If not, return a `ClassifierError::FileNotFound`.
    *   Attempt to open the file as a GDAL `Dataset`. On failure, return a `ClassifierError::GdalError`.
    *   Store the opened `Dataset` in the `UrbanClassifier` struct.
*   **Return Value:** `Result<Self, ClassifierError>`.

**5.3. `UrbanClassifier::run_classification()` (`classifier.rs`)**

*   **Action:** This is the core logic.
    1.  Validate that the input `stations_df` contains the required `station_id_col`, `lon_col`, and `lat_col`. Return an error if not.
    2.  Extract the longitude and latitude series. Create a list of coordinate points `(x, y)`.
    3.  **Coordinate Transformation:** Get the Coordinate Reference System (CRS) from the opened GDAL dataset. Create a GDAL `CoordTransform` object to transform the input WGS84 (EPSG:4326) coordinates into the raster's CRS. Apply this transformation to all points.
    4.  Access the primary raster band from `self.dataset`.
    5.  For each transformed coordinate, use the band's `read_as::<u8>(...)` method at the corresponding pixel/line offset to sample the raw LCZ code. This can be done efficiently in a tight loop.
    6.  Create a new Polars `Series` of type `UInt8` containing the raw LCZ codes.
    7.  **Apply Overrides:** If `overrides` are provided, create a new series by applying the manual values. This can be done efficiently using Polars' `zip_with` and `apply` methods.
    8.  **Create Final Columns:**
        *   `lcz_code`: The final `UInt8` series after overrides.
        *   `lcz_name`: A `Utf8` series created by mapping `lcz_code` through `Lcz::from_code` and then `lcz.full_name()`.
        *   `simple_class`: A `Utf8` series created by mapping `lcz_code` through `Lcz::from_code`, `lcz.simple_category()`, and converting the category enum to a string.
    9.  Append these three new series to a clone of the input DataFrame using `with_columns`.
    10. Return the new `Result<DataFrame, ClassifierError>`.

**6. Dependencies (`Cargo.toml`)**

The crate will depend on the following standard Rust libraries:

*   `polars`: For DataFrame manipulation.
*   `gdal`: For reading and interacting with the GeoTIFF file.
*   `geo-types`: For basic geometric types like `Point`.
*   `thiserror`: For ergonomic, standard error handling.
*   `pyo3` (with features `extension-module`): For creating the Python bindings.
*   `pyo3-polars`: For seamless conversion between Polars and Pandas DataFrames at the boundary.
*   `serde` (optional, for serializing enums if needed).

**7. Error Handling & Documentation**

*   Define a custom `Error` enum using the `thiserror` crate to represent all possible failure modes: `FileNotFound`, `ColumnNotFound`, `GdalError`, `ClassificationError`, etc.
*   All public functions must return `Result<T, ClassifierError>`.
*   Provide clear doc comments (`///`) for all public structs, enums, and functions, explaining their purpose, parameters, and return values.

**8. Python Bindings & Example Usage (`lib.rs`)**

The crate must be exposed to Python. This will be done in `src/lib.rs`.

```rust
// --- src/lib.rs ---
use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;
use polars::prelude::DataFrame;
use std::collections::HashMap;

// Import your UrbanClassifier
use crate::classifier::UrbanClassifier;

#[pyclass]
struct PyUrbanClassifier {
    classifier: UrbanClassifier,
}

#[pymethods]
impl PyUrbanClassifier {
    #[new]
    fn new(wudapt_path: &str) -> PyResult<Self> {
        let classifier = UrbanClassifier::new(wudapt_path)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(PyUrbanClassifier { classifier })
    }

    fn run_classification(
        &self,
        stations_df: PyDataFrame,
        station_id_col: &str,
        lon_col: &str,
        lat_col: &str,
        overrides: Option<HashMap<String, u8>>,
    ) -> PyResult<PyDataFrame> {
        let rust_df: DataFrame = stations_df.into();
        let result_df = self.classifier.run_classification(
            &rust_df, station_id_col, lon_col, lat_col, overrides.as_ref()
        ).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(PyDataFrame(result_df))
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn urban_classifier(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyUrbanClassifier>()?;
    Ok(())
}
```

This setup will allow a Python user to use the performant Rust code as if it were a native Python module.

---
## Rust enrichment spec:
