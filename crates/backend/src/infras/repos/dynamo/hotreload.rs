use aws_sdk_dynamodb::{error::DisplayErrorContext, types::AttributeValue, Client};
use serde_dynamo::{from_item, to_item};

use crate::application::ports::repository::{
    error::RepositoryError,
    hot_reload::{HotReloadError, HotReloadRepository},
};
use domain::{
    commands::compute::{ComputeRegion, LaunchConfig},
    errors::DomainError,
    hot_reload::{BillOptimization, HotReloadCfg, HotreloadDomain},
};
use pkg::auth::claims::Username;

// range key values - must match the `#[serde(rename = ...)]` on HotReloadCfg's variants.
const CONTEXT_BILL: &str = "bill";
const CONTEXT_LAUNCH: &str = "launch";

pub struct DDBHotReloadRepository {
    client: Client,
    table: String,
}

impl DDBHotReloadRepository {
    pub fn new(client: Client, table: String) -> Self {
        Self { client, table }
    }

    /// Fetches the single item at (`username`, `context`) - the table's full primary key.
    async fn get_one(
        &self,
        username: &Username,
        context: &str,
    ) -> Result<HotreloadDomain, HotReloadError> {
        let out = self
            .client
            .get_item()
            .table_name(&self.table)
            .key("username", AttributeValue::S(username.0.clone()))
            .key("context", AttributeValue::S(context.to_string()))
            .send()
            .await
            .map_err(|e| RepositoryError::Disconnected(DisplayErrorContext(e).to_string()))?;

        let item = out
            .item
            .ok_or_else(|| RepositoryError::NotFound(username.clone()))?;
        from_item(item).map_err(|e| RepositoryError::Serialize(e.to_string()))
    }
}

impl HotReloadRepository for DDBHotReloadRepository {
    /// Overwrites the (username, context) item `value` carries - `context` comes from
    /// which `HotReloadCfg` variant `value` holds, so writing a `Bill` record never
    /// touches that user's `Launch` record (or vice versa).
    async fn set(
        &self,
        username: &Username,
        value: &HotreloadDomain,
    ) -> Result<HotreloadDomain, DomainError> {
        let item =
            to_item(value).map_err(|e| RepositoryError::<Username>::Serialize(e.to_string()))?;
        self.client
            .put_item()
            .table_name(&self.table)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| {
                RepositoryError::<Username>::Disconnected(DisplayErrorContext(e).to_string())
            })?;
        tracing::info!("set hot-reload record for {username}");
        Ok(value.clone())
    }

    async fn bill_optimization(
        &self,
        username: &Username,
    ) -> Result<BillOptimization, HotReloadError> {
        let domain = self.get_one(username, CONTEXT_BILL).await?;
        match domain.config() {
            HotReloadCfg::Bill(b) => Ok(*b),
            HotReloadCfg::Launch(_) => Err(RepositoryError::Database(format!(
                "{username}/{CONTEXT_BILL} record is not a Bill config"
            ))),
        }
    }

    async fn launch_config(
        &self,
        username: &Username,
        region: &ComputeRegion,
    ) -> Result<LaunchConfig, HotReloadError> {
        let domain = self.get_one(username, CONTEXT_LAUNCH).await?;
        match domain.config() {
            HotReloadCfg::Launch(configs) => configs
                .iter()
                .find(|c| &c.region == region)
                .cloned()
                .ok_or_else(|| {
                    RepositoryError::Database(format!(
                        "{username}/{CONTEXT_LAUNCH} has no config for region {region}"
                    ))
                }),
            HotReloadCfg::Bill(_) => Err(RepositoryError::Database(format!(
                "{username}/{CONTEXT_LAUNCH} record is not a Launch config"
            ))),
        }
    }

    /// Every hot-reload record for `username` - one item per context (bill, launch, ...).
    async fn get(&self, username: &Username) -> Result<Vec<HotreloadDomain>, HotReloadError> {
        let out = self
            .client
            .query()
            .table_name(&self.table)
            .key_condition_expression("username = :username")
            .expression_attribute_values(":username", AttributeValue::S(username.0.clone()))
            .send()
            .await
            .map_err(|e| RepositoryError::Disconnected(DisplayErrorContext(e).to_string()))?;

        out.items
            .unwrap_or_default()
            .into_iter()
            .map(|item| from_item(item).map_err(|e| RepositoryError::Serialize(e.to_string())))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::commands::compute::ComputeCommand;
    use pkg::types::time::Second;

    // serde_dynamo's Serializer/Deserializer need to support `#[serde(flatten)]` for
    // HotreloadDomain to round-trip through a DynamoDB item at all - confirm that here
    // rather than only discovering it against a real table.
    #[test]
    fn hotreload_domain_round_trips_through_a_dynamo_item() {
        let original = HotreloadDomain::new(
            Username("alice".into()),
            HotReloadCfg::Bill(BillOptimization {
                idle_tolerance: Second(300),
                check_interval: Second(60),
                action: ComputeCommand::Stop,
            }),
        );

        let item: std::collections::HashMap<String, AttributeValue> =
            serde_dynamo::to_item(&original)
                .expect("HotreloadDomain should serialize to a dynamo item");
        assert_eq!(
            item.get("context"),
            Some(&AttributeValue::S(CONTEXT_BILL.to_string())),
            "context must be a top-level attribute so it can be the range key"
        );

        let restored: HotreloadDomain =
            from_item(item).expect("dynamo item should deserialize back to HotreloadDomain");
        assert_eq!(restored.username(), original.username());
        match restored.config() {
            HotReloadCfg::Bill(b) => assert_eq!(b.idle_tolerance.0, 300),
            HotReloadCfg::Launch(_) => panic!("expected a Bill config"),
        }
    }
}
