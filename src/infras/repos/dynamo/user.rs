use aws_sdk_dynamodb::{types::AttributeValue, Client};
use serde::{Deserialize, Serialize};
use serde_dynamo::{from_item, to_item};

use crate::{
    application::ports::repository::{
        error::RepositoryError,
        user::{UserError, UserRepository},
    },
    domain::user::{User, UserId},
    pkg::types::{strings::Email, time::Timestamp},
};

use super::table::{user_pk, USER_SK};

// Internal DynamoDB representation — PK/SK envelope wrapping domain fields.
// Kept private: callers only see User and UserRepoError.
#[derive(Debug, Serialize, Deserialize)]
struct UserItem {
    #[serde(rename = "PK")]
    pk: String,
    #[serde(rename = "SK")]
    sk: String,
    id: String,
    email: Email,
    name: String,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl From<&User> for UserItem {
    fn from(u: &User) -> Self {
        Self {
            pk: user_pk(&u.id),
            sk: USER_SK.to_string(),
            id: u.id.0.clone(),
            email: u.email.clone(),
            name: u.name.clone(),
            created_at: u.created_at.clone(),
            updated_at: u.updated_at.clone(),
        }
    }
}

impl From<UserItem> for User {
    fn from(item: UserItem) -> Self {
        User {
            id: UserId(item.id),
            email: item.email,
            name: item.name,
            created_at: item.created_at,
            updated_at: item.updated_at,
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
    async fn get(&self, id: &UserId) -> Result<User, UserError> {
        let out = self
            .client
            .get_item()
            .table_name(&self.table)
            .key("PK", AttributeValue::S(user_pk(id)))
            .key("SK", AttributeValue::S(USER_SK.to_string()))
            .send()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let item = out.item.ok_or_else(|| UserError::NotFound(id.clone()))?;
        let user_item: UserItem =
            from_item(item).map_err(|e| RepositoryError::Database(e.to_string()))?;

        let user: User = user_item.into();
        Ok(user)
    }

    async fn put(&self, user: &User) -> Result<(), UserError> {
        let av_map =
            to_item(UserItem::from(user)).map_err(|e| RepositoryError::Database(e.to_string()))?;

        self.client
            .put_item()
            .table_name(&self.table)
            .set_item(Some(av_map))
            .send()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: &UserId) -> Result<(), UserError> {
        self.client
            .delete_item()
            .table_name(&self.table)
            .key("PK", AttributeValue::S(user_pk(id)))
            .key("SK", AttributeValue::S(USER_SK.to_string()))
            .send()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}
