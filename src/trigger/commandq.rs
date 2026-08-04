use crate::{
    infras::repos::dynamo::agent_command::DDBAgentCommandRepository, pkg::types::time::Second,
};

pub struct CommandQ {
    loader: DDBAgentCommandRepository,
    interval: Second,
}

// impl CommandQ {
//     run()
// }
