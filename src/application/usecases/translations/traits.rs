use crate::application::ports::{
    clients::ctrait::{self},
    repository::rtrait::{self},
};

pub trait TLRepos: rtrait::HasTranslation {}
impl<T: rtrait::HasTranslation> TLRepos for T {}

pub trait TLClients:
    ctrait::HasTranslator + ctrait::HasRaws + ctrait::HasStorage + ctrait::HasNotification
{
}
impl<T: ctrait::HasTranslator + ctrait::HasRaws + ctrait::HasStorage + ctrait::HasNotification>
    TLClients for T
{
}
