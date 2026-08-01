pub mod command;
pub mod compute;
pub mod inference;
pub mod network;
// use serde::{Deserialize, Serialize};

// use crate::pkg::types::{id::ModelProvider, time::Timestamp, unit};

// // pub struct TODO(String);
// #[derive(Serialize, Deserialize)]
// pub struct TaskDomain {
//     pub action_id: String,
//     pub priority: u64,
//     pub status: String,
//     pub created_at: Timestamp,
//     pub ttl: Timestamp,
//     #[serde(flatten)]
//     pub action: TaskAction,
// }

// #[derive(Serialize, Deserialize)]
// #[serde(tag = "action", content = "data", rename_all = "lowercase")]
// pub enum TaskAction {
//     Generate(GenerateArgs),
//     Download(DownloadArgs),
//     Compute(ComputeArgs),
// }

// #[derive(Serialize, Deserialize)]
// struct GeneratePrompt {
//     positive: String,
//     negative: String,
// }

// #[derive(Serialize, Deserialize)]
// pub struct GenerateArgs {
//     prompts: GeneratePrompt,
//     width: unit::Px,
//     height: unit::Px,
//     steps: unit::Index1,
//     n_iter: unit::Index1,
//     seed: u32,
// }

// #[derive(Serialize, Deserialize)]
// pub struct DownloadArgs {
//     provider: ModelProvider,
//     /**
//      * e.g: s3://<bucket>/path
//      * modelId
//      * https://something/endpoint
//      */
//     path: String,
// }

// #[derive(Serialize, Deserialize)]
// pub struct ComputeArgs {}
