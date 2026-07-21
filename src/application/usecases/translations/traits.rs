use crate::application::ports::{
    clients::cc::{self},
    repository::rc::{self},
};

pub trait TLRepos: rc::HasTranslation {}
impl<T: rc::HasTranslation> TLRepos for T {}

pub trait TLClients:
    cc::HasTranslator + cc::HasRaws + cc::HasStorage + cc::HasNotification
{
}
impl<T: cc::HasTranslator + cc::HasRaws + cc::HasStorage + cc::HasNotification> TLClients for T {}
