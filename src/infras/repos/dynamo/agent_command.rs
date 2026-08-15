use aws_sdk_dynamodb::{
    operation::query::builders::QueryFluentBuilder,
    types::{AttributeAction, AttributeValue, AttributeValueUpdate},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_dynamo::aws_sdk_dynamodb_1::{from_item, to_item};

use crate::{
    application::ports::repository::{
        agent_command::{AgentCommandError, AgentCommandRepository},
        error::RepositoryError,
    },
    domain::commands::command::{ActionId, CommandDomain, CommandStage, Progression},
};

pub struct DDBAgentCommandRepository {
    client: Client,
    table: String,
}

impl DDBAgentCommandRepository {
    pub fn new(client: Client, table: String) -> Self {
        Self { client, table }
    }
}

#[derive(Debug, Clone, Copy)]
enum GSI {
    StagePriority,
}

impl From<GSI> for String {
    fn from(value: GSI) -> Self {
        let s = match value {
            GSI::StagePriority => "stage-priority-index",
        };
        String::from(s)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentCommandDDB(pub CommandDomain);

impl DDBAgentCommandRepository {
    fn query(&self) -> QueryFluentBuilder {
        self.client.query().table_name(&self.table)
    }
}

impl AgentCommandRepository for DDBAgentCommandRepository {
    async fn insert(&self, command: CommandDomain) -> Result<ActionId, AgentCommandError> {
        let action_id = command.action_id.clone();
        let item = AgentCommandDDB(command);
        let av_map = to_item(item).map_err(|e| RepositoryError::Serialize(e.to_string()))?;
        self.client
            .put_item()
            .table_name(&self.table)
            .set_item(Some(av_map))
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, debug = ?e, "put_item failed");
                RepositoryError::Disconnected(e.to_string())
            })?;
        Ok(action_id)
    }

    /**
     * returns commands that is not completed
     * including the one that's currently running
     * and those that's still queueing
     */
    async fn in_progress(&self, limit: i32) -> Result<Vec<CommandDomain>, AgentCommandError> {
        let in_progress = CommandStage::InProgress;
        let result = self
            .query()
            .key_condition_expression("stage = :stage")
            .expression_attribute_values(":stage", AttributeValue::S(in_progress.into()))
            .index_name(GSI::StagePriority)
            .limit(limit)
            .send()
            .await
            .map_err(|e| RepositoryError::Disconnected(e.to_string()))?;
        let items = result.items.unwrap_or_default();
        let serialized = items.iter().map(|item| {
            from_item::<CommandDomain>(item.clone()).map_err(|e| {
                tracing::error!(error = %e, item = ?item, "failed to deserialize DynamoDB item");
                RepositoryError::Serialize(e.to_string())
            })
        });
        let collected =
            serialized.collect::<Result<Vec<CommandDomain>, RepositoryError<ActionId>>>()?;
        Ok(collected)
    }

    async fn get(&self, id: &ActionId) -> Result<CommandDomain, AgentCommandError> {
        let item = self
            .query()
            .key_condition_expression("action_id = :action_id")
            .expression_attribute_values(":action_id", AttributeValue::S(id.clone().into()))
            .limit(1)
            .send()
            .await
            .map_err(|e| RepositoryError::Disconnected(e.to_string()))?
            .items
            .unwrap_or_default()
            .into_iter()
            .next()
            .ok_or_else(|| AgentCommandError::NotFound(id.clone()))?;

        from_item::<CommandDomain>(item).map_err(|e| RepositoryError::Serialize(e.to_string()))
    }

    async fn set_progress(
        &self,
        id: &ActionId,
        progress: &Progression,
    ) -> Result<CommandDomain, AgentCommandError> {
        if progress.is_done() {
            let mut current = self.get(id).await?;
            current.stage = CommandStage::Completed;
            let av_map =
                to_item(&current).map_err(|e| RepositoryError::Serialize(e.to_string()))?;
            tracing::info!(item = self.table, "tablename");
            tracing::info!(item = ?av_map, "updating progress to done");
            self.client
                .put_item()
                .table_name(&self.table)
                .condition_expression("attribute_exists(action_id)") // no need action_id = ??? here?
                .set_item(Some(av_map))
                .send()
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, debug = ?e, "update progres to done failed");
                    RepositoryError::Disconnected(e.to_string())
                })?;
            return Ok(current);
        } else {
            let av_map =
                to_item(progress).map_err(|e| RepositoryError::Serialize(e.to_string()))?;
            let m = AttributeValueUpdate::builder()
                .value(AttributeValue::M(av_map))
                .action(AttributeAction::Put)
                .build();
            let output = self
                .client
                .update_item()
                .table_name(&self.table)
                .condition_expression("attribute_exists(action_id)")
                .attribute_updates("progress", m)
                .send()
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, debug = ?e, "update progres to done failed");
                    RepositoryError::Disconnected(e.to_string())
                })?;
            // is output.attributes actually the whole command domain tho?
            let attributes = output.attributes();
            let result: Result<CommandDomain, AgentCommandError> = match attributes {
                None => Err(RepositoryError::NotFound(id.clone())),
                Some(x) => {
                    let domain = from_item::<CommandDomain>(x.clone())
                        .map_err(|e| RepositoryError::Serialize(e.to_string()));
                    domain
                }
            };
            result
        }
    }
}
