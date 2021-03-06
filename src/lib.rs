pub mod consts;
pub mod machinery;
pub mod process;
pub mod unix;

use serde::{Deserialize, Serialize};

pub mod spec {
    include!("spec/spec.v1alpha1.rs");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInstance {
    pub api: String,
    pub version: String,
    pub kind: String,
    pub namespace: String,
    pub id: String,
    pub spec: Box<serde_yaml::Value>,
}
