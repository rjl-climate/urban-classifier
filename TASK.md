### **Specification for Rust Urban Classifier Crate**

**1. Project Overview & Goal**

You are an expert **Rust developer** specializing in high-performance geospatial data analysis. Your task is to build a Rust crate named `urban_classifier`. This crate will be a core component of a larger analysis library and must be callable from Python.

The primary goal of this module is to classify geographic coordinates according to their urban environment, leveraging Rust's performance and safety. The classification must be robust, scientifically defensible, and based on the standard **Local Climate Zone (LCZ)** system. The crate should expose a simple, well-documented API and allow for manual validation and overrides.

**2. Essential Reading & Contextual Resources**

*(This section is language-agnostic and remains the same.)*

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
