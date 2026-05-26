use std::rc::Rc;

use rust_api::{
    application::{
        dto::translation::TranslationDTO,
        ports::{clients::cc::AllClient, repository::rc::AllRepos},
        usecases::{
            bases::Usecase,
            translations::init::{self},
        }, // usecases::translations::run::{self},
    },
    domain::errors::DomainError,
};

pub struct TranslationController<R: AllRepos, C: AllClient> {
    repo: Rc<R>,
    client: Rc<C>,
}

impl<R: AllRepos, C: AllClient> TranslationController<R, C> {
    pub fn new(repo: Rc<R>, client: Rc<C>) -> Self {
        TranslationController { repo, client }
    }

    pub async fn init(&self, params: init::Params) -> Result<TranslationDTO, DomainError> {
        let repo = self.repo.clone();
        let client = self.client.clone();
        let action = init::Init::new(repo, client, params);
        return action.exec().await;
    }
}
