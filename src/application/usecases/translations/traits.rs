use crate::application::ports::{
    clients::container as client_container, repository::container as repo_container,
};

pub trait TLRepos: repo_container::HasTranslation {}
impl<T: repo_container::HasTranslation> TLRepos for T {}

pub trait TLClients:
    client_container::HasTranslator
    + client_container::HasRaws
    + client_container::HasOutputStorage
    + client_container::HasNotification
{
}
impl<
        T: client_container::HasTranslator
            + client_container::HasRaws
            + client_container::HasOutputStorage
            + client_container::HasNotification,
    > TLClients for T
{
}
