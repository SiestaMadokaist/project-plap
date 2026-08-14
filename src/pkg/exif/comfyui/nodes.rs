use std::cell::OnceCell;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeData<V> {
    pub title: Option<String>,
    pub widgets_values: V,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ComfyNode {
    #[serde(rename = "CheckpointLoaderSimple")]
    Checkpoint(NodeData<Vec<String>>),
    #[serde(rename = "CLIPTextEncode")]
    Cliptext(NodeData<Vec<String>>),
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Default)]
struct Memo {
    positive: OnceCell<Option<String>>,
    negative: OnceCell<Option<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComfyWorkflow {
    pub id: String,
    pub nodes: Vec<ComfyNode>,
    #[serde(skip)]
    memo: Memo,
}

impl ComfyWorkflow {
    pub fn checkpoint(&self) -> Option<&String> {
        let cp = self
            .nodes
            .iter()
            .find(|x| matches!(x, ComfyNode::Checkpoint(_)))?;
        match cp {
            ComfyNode::Checkpoint(x) => x.value(),
            _ => None,
        }
    }

    pub fn positive(&self) -> &Option<String> {
        let result = self.memo.positive.get_or_init(|| {
            let positive = self
                .nodes
                .iter()
                .map(|x| x.positive_clip())
                .find(|x| matches!(x, Some(_)))??;
            return Some(positive.clone());
        });
        return result;
    }

    pub fn negative(&self) -> &Option<String> {
        let result = self.memo.negative.get_or_init(|| {
            let positive = self
                .nodes
                .iter()
                .map(|x| x.negative_clip())
                .find(|x| matches!(x, Some(_)))??;
            return Some(positive.clone());
        });
        return result;
    }
}

impl ComfyNode {
    fn negative_clip(&self) -> Option<&String> {
        match self {
            ComfyNode::Cliptext(data) => {
                let title = data.title.as_ref()?;
                let yes = title.to_lowercase().contains("negative");
                if yes {
                    let first = data.widgets_values.first();
                    first
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn positive_clip(&self) -> Option<&String> {
        match self {
            ComfyNode::Cliptext(data) => {
                let title = data.title.as_ref()?;
                let yes = title.to_lowercase().contains("positive");
                if yes {
                    let first = data.widgets_values.first();
                    first
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl NodeData<Vec<String>> {
    pub fn value(&self) -> Option<&String> {
        self.widgets_values.first()
    }
}
