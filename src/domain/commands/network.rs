use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    domain::storage::{StorageBucket, StoragePath},
    pkg::civitai,
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkAction {
    Download,
    Upload,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S3Args {
    pub bucket: StorageBucket,
    pub path: StoragePath,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "provider", content = "data")]
pub enum ModelSrc {
    #[serde(rename = "s3")]
    S3(S3Args),
    #[serde(rename = "civitai")]
    Civitai(civitai::typing::VersionId),
    // #[serde(rename = "https")]
    // Https(URL),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalArgs {
    pub forward: bool,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "provider", content = "data")]
pub enum ModelDst {
    #[serde(rename = "s3")]
    S3(S3Args),
    #[serde(rename = "localhost")]
    Local(LocalArgs),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkArgs {
    pub src: ModelSrc,
    pub dst: ModelDst,
    // prevent from being constructible aside from json deserialize
    #[serde(skip)]
    _marker: std::marker::PhantomData<()>,
}
