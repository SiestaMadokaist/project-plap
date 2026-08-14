use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    domain::storage::{StorageBucket, StoragePath},
    pkg::types::strings::URL,
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkAction {
    Download,
    Upload,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct S3Args {
    pub bucket: StorageBucket,
    pub path: StoragePath,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "provider", content = "data")]
pub enum ModelSrc {
    #[serde(rename = "s3")]
    S3(S3Args),
    #[serde(rename = "civitai")]
    Civitai(u32),
    // #[serde(rename = "https")]
    // Https(URL),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalArgs {
    pub forward: bool,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "provider", content = "data")]
pub enum ModelDst {
    #[serde(rename = "s3")]
    S3(S3Args),
    #[serde(rename = "https")]
    Local(LocalArgs),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkArgs {
    pub src: ModelSrc,
    pub dst: ModelDst,
    // prevent from being constructible aside from json deserialize
    _marker: std::marker::PhantomData<()>,
}
