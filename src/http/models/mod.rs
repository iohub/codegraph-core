pub mod build;
pub mod query;
pub mod snippet;
pub mod skeleton;
pub mod init;

pub use build::*;
pub use query::*;
pub use snippet::*;
pub use skeleton::*;
pub use init::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub success: bool,
    pub error: String,
    pub code: u16,
} 