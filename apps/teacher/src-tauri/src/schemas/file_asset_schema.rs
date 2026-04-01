use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct UploadLocalImageInput {
    pub source_path: String,
    pub biz: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UploadLocalImageOutput {
    pub relative_path: String,
    pub file_name: String,
}
