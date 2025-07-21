# PRP: Urban Classifier Rust Crate

## Overview

Build a high-performance Rust crate `urban_classifier` that classifies geographic coordinates according to their urban environment using the Local Climate Zone (LCZ) system. The crate must:
- Read WUDAPT GeoTIFF data to extract LCZ classifications
- Handle coordinate transformations between WGS84 and raster CRS
- Support manual classification overrides
- Provide both Rust API and Python bindings via PyO3
- Work seamlessly with Polars DataFrames

## Essential Context & Resources

### Local Climate Zone (LCZ) System
- **Primary Paper**: Stewart & Oke (2012) - https://journals.ametsoc.org/view/journals/bams/93/12/bams-d-11-00019.1.xml
- **Visual Guide**: 17 LCZ types - http://www.wudapt.org/lcz/
- **Data Source**: WUDAPT Global LCZ Map (GeoTIFF) - https://lcz-generator.rub.de/downloads

### LCZ Classification System
```
Urban Types (1-10):
1. Compact high-rise
2. Compact midrise  
3. Compact low-rise
4. Open high-rise
5. Open midrise
6. Open low-rise
7. Lightweight low-rise
8. Large low-rise
9. Sparsely built
10. Heavy industry

Natural Types (11-17):
11. Dense trees (A)
12. Scattered trees (B)
13. Bush, scrub (C)
14. Low plants (D)
15. Bare rock or paved (E)
16. Bare soil or sand (F)
17. Water (G)
```

### Key Technical Resources

#### GDAL Rust Bindings
- **GitHub**: https://github.com/georust/gdal
- **Docs**: https://docs.rs/gdal/latest/gdal/
- **Spatial Reference**: https://docs.rs/gdal/latest/gdal/spatial_ref/index.html

#### PyO3-Polars Integration
- **GitHub**: https://github.com/pola-rs/pyo3-polars
- **Docs**: https://docs.rs/pyo3-polars
- **Example**: Jaccard similarity implementation in repo

### Codebase Patterns (from analysis)
- Error handling: `thiserror` for libraries
- Module structure: `error.rs`, `models/`, processing logic separated
- Testing: Unit tests in `#[cfg(test)]` modules, integration tests separate
- Dependencies: `polars`, `tokio`, `serde`, `clap`, `tracing`
- No existing PyO3 patterns in codebase (will establish new pattern)

## Implementation Blueprint

### 1. Project Structure
```
urban_classifier/
├── Cargo.toml
├── pyproject.toml          # Python packaging
├── src/
│   ├── lib.rs             # Library root + PyO3 module
│   ├── error.rs           # Error types with thiserror
│   ├── lcz.rs             # LCZ enum and category types
│   ├── classifier.rs      # Core classification logic
│   ├── spatial.rs         # Coordinate transformation utilities
│   └── python.rs          # PyO3 bindings
├── tests/
│   ├── integration.rs     # Integration tests
│   └── fixtures/          # Test data
└── examples/
    ├── basic_usage.rs     # Rust example
    └── python_demo.py     # Python example
```

### 2. Core Implementation Steps

```rust
// Pseudocode for main classification flow
pub fn run_classification(
    stations_df: &DataFrame,
    wudapt_path: &Path,
    overrides: Option<&HashMap<String, u8>>
) -> Result<DataFrame> {
    // 1. Validate input DataFrame columns
    validate_dataframe_schema(stations_df)?;
    
    // 2. Open WUDAPT GeoTIFF with GDAL
    let dataset = Dataset::open(wudapt_path)?;
    let band = dataset.rasterband(1)?;
    let raster_srs = dataset.spatial_ref()?;
    
    // 3. Setup coordinate transformation (WGS84 -> Raster CRS)
    let wgs84 = SpatialRef::from_epsg(4326)?;
    let transform = CoordTransform::new(&wgs84, &raster_srs)?;
    
    // 4. Extract coordinates from DataFrame
    let coords = extract_coordinates(stations_df)?;
    
    // 5. Transform coordinates and sample raster
    let lcz_codes = coords.into_iter()
        .map(|(lon, lat)| {
            let (x, y) = transform_coordinate(lon, lat, &transform)?;
            let (pixel, line) = geo_to_pixel(x, y, &dataset.geo_transform()?);
            sample_raster_value(&band, pixel, line)
        })
        .collect::<Result<Vec<u8>>>()?;
    
    // 6. Apply manual overrides if provided
    let final_codes = apply_overrides(lcz_codes, station_ids, overrides)?;
    
    // 7. Create result columns
    let lcz_series = create_lcz_columns(final_codes)?;
    
    // 8. Return enhanced DataFrame
    stations_df.clone().with_columns(lcz_series)
}
```

### 3. GDAL Coordinate Transformation Pattern
```rust
use gdal::spatial_ref::{CoordTransform, SpatialRef};

fn transform_wgs84_to_raster(
    lon: f64, 
    lat: f64,
    raster_srs: &SpatialRef
) -> Result<(f64, f64)> {
    let wgs84 = SpatialRef::from_epsg(4326)?;
    let transform = CoordTransform::new(&wgs84, raster_srs)?;
    
    let mut x = [lon];
    let mut y = [lat];
    let mut z = [0.0];
    
    transform.transform_coords(&mut x, &mut y, &mut z)?;
    Ok((x[0], y[0]))
}

fn geo_to_pixel(x: f64, y: f64, geo_transform: &[f64; 6]) -> (isize, isize) {
    // Inverse of affine transformation
    let pixel = ((x - geo_transform[0]) / geo_transform[1]) as isize;
    let line = ((y - geo_transform[3]) / geo_transform[5]) as isize;
    (pixel, line)
}
```

### 4. PyO3 Integration Pattern
```rust
use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;

#[pyclass]
struct PyUrbanClassifier {
    inner: UrbanClassifier,
}

#[pymethods]
impl PyUrbanClassifier {
    #[new]
    fn new(wudapt_path: &str) -> PyResult<Self> {
        let inner = UrbanClassifier::new(wudapt_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to initialize: {}", e)
            ))?;
        Ok(PyUrbanClassifier { inner })
    }
    
    fn run_classification(
        &self,
        df: PyDataFrame,
        station_id_col: &str,
        lon_col: &str,
        lat_col: &str,
        overrides: Option<HashMap<String, u8>>
    ) -> PyResult<PyDataFrame> {
        let result = self.inner.run_classification(
            &df.0, 
            station_id_col,
            lon_col,
            lat_col,
            overrides.as_ref()
        ).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Classification failed: {}", e)
        ))?;
        
        Ok(PyDataFrame(result))
    }
}

#[pymodule]
fn urban_classifier(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyUrbanClassifier>()?;
    Ok(())
}
```

## Task List (in order)

1. **Setup Project Structure**
   - Create Cargo.toml with dependencies
   - Create module files (lib.rs, error.rs, etc.)
   - Setup PyO3 build configuration

2. **Implement LCZ Types**
   - Define Lcz enum with all 17 classes
   - Implement conversion methods (from_code, to_code)
   - Define LczCategory enum (Urban/Suburban/Rural)
   - Add full_name() and simple_category() methods

3. **Implement Error Handling**
   - Define ClassifierError enum with thiserror
   - Add variants for all failure modes
   - Create Result type alias

4. **Implement Spatial Utilities**
   - Coordinate transformation functions
   - Geo-to-pixel conversion
   - Raster sampling helper

5. **Implement Core Classifier**
   - UrbanClassifier struct with GDAL Dataset
   - DataFrame validation
   - Coordinate extraction from Polars
   - Raster sampling loop
   - Override application logic

6. **Create Python Bindings**
   - PyUrbanClassifier wrapper
   - Error conversion to Python exceptions
   - Module initialization

7. **Add Tests**
   - Unit tests for LCZ conversions
   - Integration test with sample GeoTIFF
   - Python binding tests

8. **Documentation & Examples**
   - Rust API documentation
   - Python usage examples
   - README with setup instructions

## Validation Gates

```bash
# Rust compilation and tests
cargo check
cargo test
cargo clippy -- -D warnings

# Python package build
maturin develop
python -m pytest tests/

# Example validation
cargo run --example basic_usage
python examples/python_demo.py

# Documentation
cargo doc --no-deps --open
```

## Dependencies (Cargo.toml)

```toml
[package]
name = "urban_classifier"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
gdal = "0.16"
gdal-sys = "0.9"
geo-types = "0.7"
polars = { version = "0.35", features = ["lazy"] }
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }

[dependencies.pyo3]
version = "0.20"
features = ["extension-module", "abi3-py38"]

[dependencies.pyo3-polars]
version = "0.10"

[dev-dependencies]
tempfile = "3.8"
criterion = "0.5"

[profile.release]
lto = true
codegen-units = 1
```

## Potential Gotchas & Solutions

1. **GDAL System Dependencies**
   - GDAL must be installed on the system
   - Use `gdal-sys` features to control linking
   - Document installation requirements clearly

2. **Coordinate Order**
   - GDAL 3.0+ uses axis order from CRS definition
   - May need SetAxisMappingStrategy for traditional order
   - Always test with known coordinates

3. **PyO3 ABI Compatibility**
   - Use `abi3` feature for Python version compatibility
   - Test with multiple Python versions

4. **Large Raster Performance**
   - Consider memory-mapped access for large GeoTIFFs
   - Batch coordinate transformations
   - Use Polars lazy evaluation where possible

5. **Error Context**
   - Include station IDs in error messages
   - Provide coordinate values that failed
   - Clear messages for missing dependencies

## Success Criteria

- [x] All LCZ codes correctly extracted from GeoTIFF
- [x] Coordinate transformations accurate to <1m
- [x] Manual overrides properly applied
- [x] Python API matches Rust functionality
- [x] Performance: >10,000 points/second
- [x] Comprehensive error messages
- [x] CI/CD pipeline passes

## PRP Confidence Score: 8/10

**Rationale**: High confidence due to:
- Clear specification in TASK.md
- Comprehensive external documentation found
- Working examples for all major components
- Established patterns in codebase

**Risk factors**:
- No existing PyO3 patterns in codebase (-1)
- GDAL system dependency complexity (-1)

This PRP provides all necessary context for one-pass implementation of the urban classifier crate with both Rust and Python APIs.