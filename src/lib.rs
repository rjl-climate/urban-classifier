pub mod error;
pub mod lcz;
pub mod spatial;
pub mod classifier;

#[cfg(feature = "python")]
pub mod python;

pub use error::ClassifierError;
pub use lcz::{Lcz, LczCategory};
pub use classifier::UrbanClassifier;

// Re-export for Python bindings
#[cfg(feature = "python")]
pub use python::urban_classifier_module;