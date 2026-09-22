use std::cell::OnceCell;

use serde::{Deserialize, Serialize};

use crate::comfy::workflow::web::nodes::WebNode;

pub mod nodes;

#[derive(Debug, Clone, Default)]
struct Memo {
    positive: OnceCell<Option<String>>,
    negative: OnceCell<Option<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExportWeb {
    pub id: String,
    pub nodes: Vec<WebNode>,
    #[serde(skip)]
    memo: Memo,
}

impl ExportWeb {
    pub fn checkpoint(&self) -> Option<&str> {
        let cp = self.nodes.iter().find(|x| x.is_checkpoint())?;
        match cp {
            WebNode::Checkpoint(x) => x.value(),
            WebNode::UnetLoader(x) => x.value(),
            _ => None,
        }
    }

    pub fn positive(&self) -> Option<&str> {
        let result = self.memo.positive.get_or_init(|| {
            let positive: &String = self
                .nodes
                .iter()
                .map(|x| x.positive_clip())
                .find(|x| x.is_some())??;
            Some(positive.trim().into())
        });
        result.as_deref()
    }

    pub fn negative(&self) -> Option<&str> {
        let result = self.memo.negative.get_or_init(|| {
            let negative: &String = self
                .nodes
                .iter()
                .map(|x| x.negative_clip())
                .find(|x| x.is_some())??;
            Some(negative.trim().into())
        });
        result.as_deref()
    }
}
