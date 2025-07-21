use thiserror::Error;

pub type Result<T> = std::result::Result<T, ClassifierError>;

#[derive(Error, Debug)]
pub enum ClassifierError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("GDAL error: {message}")]
    GdalError { message: String },

    #[error("Column not found in DataFrame: {column}")]
    ColumnNotFound { column: String },

    #[error("Invalid coordinate: lon={lon}, lat={lat}")]
    InvalidCoordinate { lon: f64, lat: f64 },

    #[error("Coordinate transformation failed: {message}")]
    CoordinateTransform { message: String },

    #[error("Raster sampling failed at pixel ({pixel}, {line}): {message}")]
    RasterSampling {
        pixel: isize,
        line: isize,
        message: String,
    },

    #[error("DataFrame schema validation failed: {message}")]
    SchemaValidation { message: String },

    #[error("Override application failed for station {station_id}: {message}")]
    OverrideApplication {
        station_id: String,
        message: String,
    },

    #[error("Polars error: {0}")]
    Polars(#[from] polars::error::PolarsError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<gdal::errors::GdalError> for ClassifierError {
    fn from(err: gdal::errors::GdalError) -> Self {
        ClassifierError::GdalError {
            message: err.to_string(),
        }
    }
}