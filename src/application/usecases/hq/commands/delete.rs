use std::rc::Rc;

use crate::{
    application::{
        ports::repository::container::HasAgentCommand,
        usecases::hq::commands::cp_model::CPModelRepos,
    },
    domain::commands::command::ActionId,
    pkg::{macros::trait_repos, types::time::Timestamp},
};

pub struct Payload {
    action_id: ActionId,
}

trait_repos!(DeleteCommandRepos, HasAgentCommand);

pub struct DeleteCommand<R: DeleteCommandRepos> {
    repos: Rc<R>,
    payload: Payload,
}

impl<R: DeleteCommandRepos> DeleteCommand<R> {
    pub fn new(repos: Rc<R>, payload: Payload) -> Self {
        Self { repos, payload }
    }
}
