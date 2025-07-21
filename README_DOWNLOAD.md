# WUDAPT Global LCZ Map Downloader

This utility downloads the Global Local Climate Zone (LCZ) Map from WUDAPT and places it in a location where the `urban_classifier` can find it automatically.

## Installation

Build the utility:

```bash
# Build the download utility
cargo build --bin download_wudapt --release

# The binary will be available at:
./target/release/download_wudapt
```

## Quick Start

Simply run the downloader with no arguments:

```bash
./target/release/download_wudapt
```

This will:
1. Download the latest Global LCZ Map (Version 3, ~4GB)
2. Save it as `wudapt_lcz_global.tif` in the current directory
3. Verify the file is a valid GeoTIFF
4. Show you how to use it with `urban_classifier`

## Command Line Options

```bash
WUDAPT Global LCZ Map Downloader 0.1.0
Downloads the Global Local Climate Zone (LCZ) map from WUDAPT for use with urban_classifier

USAGE:
    download_wudapt [OPTIONS]

OPTIONS:
    -o, --output <FILE>   Output file path (default: ./wudapt_lcz_global.tif)
    -u, --url <URL>       Custom download URL (uses WUDAPT official URLs by default)
    -f, --force           Force download even if file already exists
        --list-locations  List default download locations
    -h, --help            Print help
    -V, --version         Print version
```

## Examples

### Basic Download
```bash
# Download to current directory (default)
./target/release/download_wudapt
```

### Custom Output Location
```bash
# Download to specific path
./target/release/download_wudapt --output /data/lcz_map.tif

# Download to data directory
./target/release/download_wudapt --output data/wudapt_lcz_global.tif
```

### Force Re-download
```bash
# Re-download even if file exists
./target/release/download_wudapt --force
```

### List Default Locations
```bash
# See where the utility will look for/place files
./target/release/download_wudapt --list-locations
```

## Default Download Locations

The utility will place the downloaded file in one of these locations (in order of preference):

1. `./wudapt_lcz_global.tif` (current directory)
2. `./data/wudapt_lcz_global.tif` (data subdirectory)
3. `/tmp/wudapt_lcz_global.tif` (temporary directory)
4. `~/.cache/urban_classifier/wudapt_lcz_global.tif` (user cache)

## Data Sources

The utility downloads from these sources (in order):

1. **LCZ Generator v3** (Primary): `https://lcz-generator.rub.de/cogs/lcz_filter_v3_cog.tif`
   - Cloud-Optimized GeoTIFF format
   - Latest version with morphological filtering applied

2. **Zenodo v3** (Backup): `https://zenodo.org/records/6364594/files/lcz_filter_v3.tif`
   - Academic repository with DOI
   - Same data, standard GeoTIFF format

3. **LCZ Generator v2** (Fallback): `https://lcz-generator.rub.de/cogs/lcz_filter_v2_cog.tif`
   - Previous version if v3 is unavailable

## File Information

- **Size**: ~4GB
- **Resolution**: 100m spatial resolution
- **Projection**: WGS84 (EPSG:4326)
- **Coverage**: Global
- **Year**: 2018 (nominal)
- **Format**: GeoTIFF with embedded LCZ color scheme

## Using with Urban Classifier

Once downloaded, you can use the file with `urban_classifier`:

### Rust
```rust
use urban_classifier::UrbanClassifier;

let classifier = UrbanClassifier::new("wudapt_lcz_global.tif")?;
let results = classifier.run_classification(
    &stations_df, "station_id", "longitude", "latitude", None
)?;
```

### Python
```python
import urban_classifier

classifier = urban_classifier.PyUrbanClassifier("wudapt_lcz_global.tif")
results = classifier.run_classification(
    df, "station_id", "longitude", "latitude"
)
```

## Troubleshooting

### Download Fails
- Check internet connection
- Try again later (servers may be temporarily unavailable)
- Use `--url` to specify a custom download URL
- Download manually and place in one of the default locations

### File Verification Fails
- Use `--force` to re-download
- Check available disk space (~5GB required)
- Ensure write permissions to output directory

### Large File Size
The Global LCZ Map is approximately 4GB. Ensure you have:
- Sufficient disk space (5+ GB free)
- Stable internet connection
- Patience (download may take 10-60 minutes depending on connection)

## Citation

When using this data, please cite:

```
Demuzere, M., Kittner, J., Martilli, A., Mills, G., Moede, C., Stewart, I. D., 
van Vliet, J., and Bechtel, B.: A global map of local climate zones to support 
earth system modelling and urban-scale environmental science, Earth Syst. Sci. Data, 
14, 3835–3873, https://doi.org/10.5194/essd-14-3835-2022, 2022.
```

## License

This utility is provided under the same license as the `urban_classifier` crate.
The WUDAPT Global LCZ Map data is provided by the WUDAPT project and has its own licensing terms.