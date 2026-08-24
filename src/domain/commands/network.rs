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

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use crate::{
        displayable,
        domain::commands::network::{ModelDst, ModelSrc, NetworkArgs},
    };

    #[derive(Debug, thiserror::Error, Serialize)]
    pub enum E {
        Serialize(String),
        Openfile(String),
        Misread(String),
    }
    displayable!(E);

    fn is_s32local(path: &str) -> Result<(), E> {
        // s3 to localhost
        let buffer = std::fs::read(path).map_err(|e| E::Openfile(e.to_string()))?;
        let command: NetworkArgs =
            serde_json::from_slice(&buffer).map_err(|e| E::Serialize(e.to_string()))?;
        match command.src {
            ModelSrc::S3(src) => match command.dst {
                ModelDst::Local(dst) => {
                    assert_eq!(&src.bucket.0, "test-bucket");
                    assert_eq!(&src.path.0, "path/from/src");
                    assert_eq!(&dst.forward, &false);
                    assert_eq!(&dst.path, "path/to/dst");
                }
                _ => return Err(E::Misread("dst should be local".into())),
            },
            _ => return Err(E::Misread("src should be s3".into())),
        }
        Ok(())
    }

    fn is_civit2local(path: &str) -> Result<(), E> {
        let buffer = std::fs::read(path).map_err(|e| E::Openfile(e.to_string()))?;
        let command: NetworkArgs =
            serde_json::from_slice(&buffer).map_err(|e| E::Serialize(e.to_string()))?;
        match command.src {
            ModelSrc::Civitai(src) => match command.dst {
                ModelDst::Local(dst) => {
                    assert_eq!(src.0, 132);
                    assert_eq!(&dst.forward, &true);
                    assert_eq!(&dst.path, "path/to/dst");
                }
                _ => return Err(E::Misread("dst should be local".into())),
            },
            _ => return Err(E::Misread("src should be civitai".into())),
        }
        Ok(())
    }

    #[test]
    fn test_s32local() -> Result<(), E> {
        let ok = is_s32local("./samples/inputs/jsons/domain/commands/network1.json");
        assert!(matches!(ok, Ok(_)));
        let not_ok = is_s32local("./samples/inputs/jsons/domain/commands/network2.json");
        assert!(matches!(not_ok, Err(_)));
        Ok(())
    }

    #[test]
    fn test_civit2local() -> Result<(), E> {
        let not_ok = is_civit2local("./samples/inputs/jsons/domain/commands/network1.json");
        assert!(matches!(not_ok, Err(_)));
        let ok = is_civit2local("./samples/inputs/jsons/domain/commands/network2.json");
        assert!(matches!(ok, Ok(_)));
        Ok(())
    }
}
