use aws_sdk_dynamodb::{
    types::{AttributeValue, ReturnValue},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_dynamo::{from_item, to_item};

use crate::application::ports::repository::{
    error::RepositoryError,
    user::{UserError, UserRepository},
};
use domain::user::User;
use pkg::{
    auth::{claims::Username, ecdsa::AddressETH},
    types::time::{Timestamp, TimestampMS},
};

/// GSI whose partition key is the `address` attribute.
const ADDRESS_INDEX: &str = "address";

/// domain keeps wall-clock seconds; the DynamoDB record stores milliseconds.
fn to_ms(ts: Timestamp) -> TimestampMS {
    TimestampMS(ts.0 * 1_000)
}
fn to_secs(ms: TimestampMS) -> Timestamp {
    Timestamp(ms.0 / 1_000)
}

#[derive(Debug, Serialize, Deserialize)]
struct UserItem {
    // partition key
    username: Username,
    // GSI `address` partition key
    address: AddressETH,
    #[serde(rename = "createdAt")]
    created_at: TimestampMS,
    // absent until activated / first login - kept absent (not NULL) so the
    // `attribute_not_exists(lastLogin)` guard in `login` works.
    #[serde(
        rename = "activatedAt",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    activated_at: Option<TimestampMS>,
    #[serde(rename = "lastLogin", default, skip_serializing_if = "Option::is_none")]
    last_login: Option<TimestampMS>,
}

impl From<&User> for UserItem {
    fn from(u: &User) -> Self {
        Self {
            username: u.username.clone(),
            address: u.address.clone(),
            created_at: to_ms(u.created_at),
            activated_at: u.activated_at.map(to_ms),
            last_login: u.last_login.map(to_ms),
        }
    }
}

impl From<UserItem> for User {
    fn from(item: UserItem) -> Self {
        User {
            username: item.username,
            address: item.address,
            created_at: to_secs(item.created_at),
            activated_at: item.activated_at.map(to_secs),
            last_login: item.last_login.map(to_secs),
        }
    }
}

pub struct DDBUserRepository {
    client: Client,
    table: String,
}

impl DDBUserRepository {
    pub fn new(client: Client, table: String) -> Self {
        Self { client, table }
    }
}

impl UserRepository for DDBUserRepository {
    async fn get(&self, id: &Username) -> Result<User, UserError> {
        let out = self
            .client
            .get_item()
            .table_name(&self.table)
            .key("username", AttributeValue::S(id.0.clone()))
            .send()
            .await
            .map_err(|e| RepositoryError::Disconnected(e.to_string()))?;

        let item = out
            .item
            .ok_or_else(|| RepositoryError::NotFound(id.clone()))?;
        let user_item: UserItem =
            from_item(item).map_err(|e| RepositoryError::Serialize(e.to_string()))?;
        Ok(user_item.into())
    }

    async fn put(&self, user: &User) -> Result<(), UserError> {
        let av_map =
            to_item(UserItem::from(user)).map_err(|e| RepositoryError::Serialize(e.to_string()))?;

        self.client
            .put_item()
            .table_name(&self.table)
            .set_item(Some(av_map))
            .send()
            .await
            .map_err(|e| RepositoryError::Disconnected(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: &Username) -> Result<(), UserError> {
        self.client
            .delete_item()
            .table_name(&self.table)
            .key("username", AttributeValue::S(id.0.clone()))
            .send()
            .await
            .map_err(|e| RepositoryError::Disconnected(e.to_string()))?;

        Ok(())
    }

    async fn find(&self, address: &AddressETH) -> Result<User, UserError> {
        let item = self
            .client
            .query()
            .table_name(&self.table)
            .index_name(ADDRESS_INDEX)
            .key_condition_expression("address = :address")
            .expression_attribute_values(":address", AttributeValue::S(address.0.clone()))
            .limit(1)
            .send()
            .await
            .map_err(|e| RepositoryError::Disconnected(e.to_string()))?
            .items
            .unwrap_or_default()
            .into_iter()
            .next()
            .ok_or_else(|| RepositoryError::Conflict(address.0.clone()))?;

        let user_item: UserItem =
            from_item(item).map_err(|e| RepositoryError::Serialize(e.to_string()))?;
        Ok(user_item.into())
    }

    async fn login(&self, address: &AddressETH, iat: Timestamp) -> Result<User, UserError> {
        let user = self.find(address).await?;
        if user.activated_at.is_none() {
            return Err(RepositoryError::Conflict(format!(
                "user {} is not activated",
                user.username
            )));
        }

        let iat_ms = to_ms(iat).0;
        let out = self
            .client
            .update_item()
            .table_name(&self.table)
            .key("username", AttributeValue::S(user.username.0.clone()))
            // monotonic last_login: reject a stale or replayed `iat`. Enforced here rather
            // than read-then-write so two concurrent logins can't both pass.
            .condition_expression("attribute_not_exists(lastLogin) OR lastLogin < :iat")
            .update_expression("SET lastLogin = :iat")
            .expression_attribute_values(":iat", AttributeValue::N(iat_ms.to_string()))
            .return_values(ReturnValue::AllNew)
            .send()
            .await
            .map_err(|e| match e.as_service_error() {
                Some(se) if se.is_conditional_check_failed_exception() => {
                    RepositoryError::Conflict(format!(
                        "login timestamp {} is not newer than the last login",
                        iat.0
                    ))
                }
                _ => RepositoryError::Disconnected(e.to_string()),
            })?;

        let item = out
            .attributes
            .ok_or_else(|| RepositoryError::Serialize("update returned no attributes".into()))?;
        let user_item: UserItem =
            from_item(item).map_err(|e| RepositoryError::Serialize(e.to_string()))?;
        Ok(user_item.into())
    }
}
