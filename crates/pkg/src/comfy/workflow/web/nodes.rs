use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeData<V> {
    pub title: Option<String>,
    pub widgets_values: V,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum WebNode {
    #[serde(rename = "CheckpointLoaderSimple")]
    Checkpoint(NodeData<Vec<String>>),
    #[serde(rename = "CLIPTextEncode")]
    Cliptext(NodeData<Vec<String>>),
    #[serde(rename = "UNETLoader")]
    UnetLoader(NodeData<Vec<String>>),
    #[serde(other)]
    Other,
}

impl WebNode {
    pub fn is_checkpoint(&self) -> bool {
        if matches!(self, WebNode::Checkpoint(_)) {
            return true;
        }
        if matches!(self, WebNode::UnetLoader(_)) {
            return true;
        }
        false
    }

    pub(super) fn negative_clip(&self) -> Option<&String> {
        match self {
            WebNode::Cliptext(data) => {
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

    pub(super) fn positive_clip(&self) -> Option<&String> {
        match self {
            WebNode::Cliptext(data) => {
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
    pub fn value(&self) -> Option<&str> {
        self.widgets_values.first().map(|x| x.as_str())
    }
}
