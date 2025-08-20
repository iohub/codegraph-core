pub mod detection;
pub mod rust;
pub mod java;
pub mod python;
pub mod cpp;
pub mod typescript;
pub mod javascript;

pub use crate::codegraph::analyzers::language_adapters::*;
pub use crate::codegraph::analyzers::get_language_id_by_filename; 