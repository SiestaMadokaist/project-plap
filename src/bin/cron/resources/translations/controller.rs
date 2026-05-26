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
    _repo: Rc<R>,
    _client: Rc<C>,
}

impl<R: AllRepos, C: AllClient> TranslationController<R, C> {
    pub fn new(_repo: Rc<R>, _client: Rc<C>) -> Self {
        TranslationController { _repo, _client }
    }

    fn repo(&self) -> Rc<R> {
        return self._repo.clone();
    }

    fn client(&self) -> Rc<C> {
        return self._client.clone();
    }

    pub async fn init(&self, params: init::Params) -> Result<TranslationDTO, DomainError> {
        let action = init::Init::new(self.repo(), self.client(), params);
        return action.exec().await;
    }
}
